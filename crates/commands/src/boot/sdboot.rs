#[cfg(test)]
mod tests;

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use toolbox_cli::Exit;
use toolbox_cli::args::Read;
use toolbox_core::Snapshot;
use toolbox_core::features::boot::view::EntryRow;
use toolbox_plan::sdboot::SdBootMenu;
use toolbox_state::boot::KernelCmdline;
use toolbox_system::{BootMenuSource as _, MenuRead, SystemBootMenu};

use super::entry_listing;
use crate::settings::on_off_text;
use crate::{json, print, unavailable};

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
