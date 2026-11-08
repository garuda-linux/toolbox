#[cfg(test)]
mod tests;

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use clap::ValueEnum as _;
use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{BootKey, BootSet, Read};
use toolbox_core::Snapshot;
use toolbox_core::domains::{CurrentLoader, LoaderSettingsDomain};
use toolbox_core::features::boot::view::EntryRow;
use toolbox_plan::LoaderDefault;
use toolbox_plan::sdboot::{LoaderSetting, LoaderTimeout, SdBootMenu};
use toolbox_state::boot::KernelCmdline;
use toolbox_system::{BootMenuSource as _, MenuRead, SystemBootMenu};

use super::entry_listing;
use crate::act::run;
use crate::change::{Refused, fresh};
use crate::settings::on_off_text;
use crate::{json, print, unavailable};

/// The same limit as GRUB's timeout on the page.
const MAX_TIMEOUT: u32 = 300;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct SdBootView {
    pub loader: &'static str,
    pub timeout: Option<String>,
    pub default: Option<String>,
    pub editor: Option<bool>,
    pub auto_entries: Option<bool>,
    pub auto_firmware: Option<bool>,
    pub parameters: Option<Vec<String>>,
}

pub fn view(menu: &SdBootMenu, cmdline: Option<&KernelCmdline>) -> SdBootView {
    let settings = &menu.settings;

    SdBootView {
        loader: "systemd-boot",
        timeout: settings.timeout.map(|timeout| timeout.to_string()),
        default: settings
            .default
            .as_ref()
            .map(|default| default.as_str().to_owned()),
        editor: settings.editor,
        auto_entries: settings.auto_entries,
        auto_firmware: settings.auto_firmware,
        parameters: cmdline.map(|cmdline| cmdline.0.iter().map(ToString::to_string).collect()),
    }
}

/// The rows `boot set default` may name: a snapshot entry goes when
/// snapper deletes it.
pub fn rows(menu: &SdBootMenu) -> Vec<EntryRow> {
    menu.choosable()
        .map(|entry| EntryRow {
            entry: entry.id.as_str().to_owned(),
            title: entry.title.clone(),
            parent: String::new(),
        })
        .collect()
}

pub fn show(read: Read, snapshot: &Snapshot, out: &mut dyn Write) -> Exit {
    let Some(menu) = read_menu() else {
        return unavailable("the systemd-boot menu could not be read", out);
    };

    let view = view(&menu, snapshot.part::<KernelCmdline>());
    let text = if read.json {
        json(&view)
    } else {
        let unset = |value: Option<bool>| value.map_or("", on_off_text);
        let parameters = view
            .parameters
            .as_ref()
            .map_or_else(String::new, |list| list.join(" "));

        let paint = crate::paint();

        let facts = paint.facts(
            "Boot menu (systemd-boot)",
            &[
                ("default", view.default.as_deref().unwrap_or("")),
                ("timeout", view.timeout.as_deref().unwrap_or("")),
                ("editor", unset(view.editor)),
                ("auto-entries", unset(view.auto_entries)),
                ("auto-firmware", unset(view.auto_firmware)),
                ("parameters", &parameters),
            ],
        );

        format!("{facts}\n")
    };

    write!(out, "{text}").ok();
    Exit::Success
}

pub fn entries(read: Read, out: &mut dyn Write) -> Exit {
    match read_menu() {
        Some(menu) => print(read, &rows(&menu), out, entry_listing),
        None => unavailable("the systemd-boot menu could not be read", out),
    }
}

fn read_menu() -> Option<SdBootMenu> {
    let source = SystemBootMenu::system(Arc::new(crate::root::helper()));

    match source.menu(&Arc::new(AtomicBool::new(false))) {
        MenuRead::SdBoot(menu) => Some(menu),
        other @ (MenuRead::Known(_)
        | MenuRead::Missing
        | MenuRead::Refused(_)
        | MenuRead::Unknown(_)
        | MenuRead::Cancelled) => {
            tracing::warn!("the systemd-boot menu was not read: {other:?}");
            None
        }
    }
}

pub fn set(args: &BootSet, snapshot: &Snapshot, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    let Some(menu) = read_menu() else {
        return unavailable("the systemd-boot menu could not be read", out);
    };

    let planned = loader_setting(args.key, &args.value, &menu).and_then(|setting| {
        let mut state = snapshot.clone();

        state.insert(CurrentLoader(menu.settings.clone()));
        fresh(|changes| {
            LoaderSettingsDomain::want(changes, setting, &state);
            Ok(())
        })
    });

    run(planned, args.confirm, None, runtime, out)
}

/// # Errors
///
/// [`Refused::Rule`] for a value the setting does not take, an entry the
/// menu lacks, or a setting systemd-boot does not have.
pub fn loader_setting(
    key: BootKey,
    value: &str,
    menu: &SdBootMenu,
) -> Result<LoaderSetting, Refused> {
    let switch = |word: &str| match word {
        "none" => Ok(None),
        other => crate::settings::on_off(other)
            .map(Some)
            .map_err(Refused::Rule),
    };

    match key {
        BootKey::Default => default(value, menu).map(LoaderSetting::Default),
        BootKey::Timeout => timeout(value).map(LoaderSetting::Timeout),
        BootKey::Remember => crate::settings::on_off(value)
            .map_err(Refused::Rule)
            .map(|on| LoaderSetting::Default(on.then(LoaderDefault::saved))),
        BootKey::Editor => switch(value).map(LoaderSetting::Editor),
        BootKey::AutoEntries => switch(value).map(LoaderSetting::AutoEntries),
        BootKey::AutoFirmware => switch(value).map(LoaderSetting::AutoFirmware),
        BootKey::Submenus | BootKey::OsProber | BootKey::Background | BootKey::Theme => {
            Err(Refused::Rule(format!(
                "`{}` is a GRUB setting, this system starts with systemd-boot",
                key_name(key)
            )))
        }
        BootKey::Quiet
        | BootKey::Splash
        | BootKey::MitigationsOff
        | BootKey::Sysrq
        | BootKey::Plymouth => Err(Refused::Rule(format!(
            "`{}` is not changed on systemd-boot yet",
            key_name(key)
        ))),
    }
}

fn default(value: &str, menu: &SdBootMenu) -> Result<Option<LoaderDefault>, Refused> {
    match value {
        "none" => Ok(None),
        "saved" => Ok(Some(LoaderDefault::saved())),
        id => menu
            .choosable()
            .find(|entry| entry.id.as_str() == id)
            .map(|entry| Some(entry.default_pattern()))
            .ok_or_else(|| {
                Refused::Rule(format!("the menu has no entry `{id}`, see `boot entries`"))
            }),
    }
}

fn timeout(value: &str) -> Result<Option<LoaderTimeout>, Refused> {
    match value {
        "none" => Ok(None),
        "menu-force" => Ok(Some(LoaderTimeout::MenuForce)),
        "menu-hidden" => Ok(Some(LoaderTimeout::MenuHidden)),
        seconds => seconds
            .parse()
            .ok()
            .filter(|seconds| *seconds <= MAX_TIMEOUT)
            .map(|seconds| Some(LoaderTimeout::Seconds(seconds)))
            .ok_or_else(|| {
                Refused::Rule(format!(
                    "`{value}` is no number of seconds 0 to {MAX_TIMEOUT}, menu-force, menu-hidden or none"
                ))
            }),
    }
}

fn key_name(key: BootKey) -> String {
    key.to_possible_value()
        .map_or_else(String::new, |value| value.get_name().to_owned())
}
