//! The Boot Options page's commands, the Kernels page's default kernel
//! and its DKMS builds. Every boot change goes through the page's own
//! [`edit`], so a bad value names the rule the page names.

mod listing;
mod sdboot;
#[cfg(test)]
mod tests;

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::{
    BootCommand, BootKey, BootSet, DkmsBuild, DkmsCommand, KernelArgs, ParamCommand, Read,
};
use toolbox_core::domains::DkmsBuilds;
use toolbox_core::features::boot::edit::{Edit, EditProblem, edit};
use toolbox_core::features::boot::{Flag, shown};
use toolbox_core::features::kernels::{
    DkmsEntry, DkmsState, KernelReleases, LinuxPackages, builds, kernels as list_kernels,
};
use toolbox_core::{ChangeSet, Queued, Snapshot};
use toolbox_plan::{DkmsBuild as Build, GrubMenu, PackageName, Plan};
use toolbox_state::boot::BootLoader;
use toolbox_system::{
    BootMenuSource as _, Dkms, DkmsSource as _, MenuRead, ProbeError, SystemBootMenu,
};

pub use self::listing::{entry_listing, module_listing};
use super::act::run;
use super::change::{Refused, fresh, page_edit, why};
use super::filter::print_all;
use super::names::kernel_names;
use super::out::failure;
use super::settings::on_off_text;
use super::{json, print, system, unavailable};

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct BootView {
    pub default: String,
    pub remember: bool,
    pub timeout: Option<i64>,
    pub parameters: Option<Vec<String>>,
    pub submenus: bool,
    pub os_prober: bool,
    pub background: String,
    pub theme: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModuleRow {
    #[serde(rename = "id")]
    pub module: String,
    pub version: String,
    pub kernel: String,
    /// dkms's own word: `added`, `built`, `installed` or `broken`.
    pub state: &'static str,
}

pub fn command(command: BootCommand, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    let snapshot = system();
    let sdboot = snapshot.part::<BootLoader>() == Some(&BootLoader::SystemdBoot);

    match command {
        BootCommand::Show(read) if sdboot => sdboot::show(read, &snapshot, out),
        BootCommand::Entries(read) if sdboot => sdboot::entries(read, out),
        BootCommand::Show(read) => show(read, &snapshot, out),
        BootCommand::Entries(read) => entries(read, out),
        BootCommand::Set(args) if sdboot => sdboot::set(&args, &snapshot, runtime, out),
        BootCommand::Set(args) => {
            let planned = set(&args, &snapshot);
            run(planned, args.confirm, None, runtime, out)
        }
        BootCommand::Param(ParamCommand::Add(args)) => {
            let planned = boot_edit(Edit::AddParameter(args.parameter.to_string()), &snapshot);
            run(planned, args.confirm, None, runtime, out)
        }
        BootCommand::Param(ParamCommand::Remove(args)) => {
            let planned = boot_edit(Edit::RemoveParameter(args.parameter.to_string()), &snapshot);
            run(planned, args.confirm, None, runtime, out)
        }
    }
}

pub fn kernel_default(args: &KernelArgs, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    let snapshot = system();
    if snapshot.part::<BootLoader>() == Some(&BootLoader::SystemdBoot) {
        return sdboot::kernel_default(args, &snapshot, runtime, out);
    }

    let Some(menu) = read_menu() else {
        return unavailable("the boot menu could not be read", out);
    };

    let planned = default_kernel(&args.kernel, &menu, &snapshot);
    run(planned, args.confirm, None, runtime, out)
}

pub fn dkms(command: DkmsCommand, runtime: &Runtime, out: &mut dyn Write) -> Exit {
    let quiet = !matches!(&command, DkmsCommand::List(read) if !read.json);
    let entries = match super::spinner::while_reading(quiet, || Dkms::system().status()) {
        Ok(entries) => entries,
        Err(ProbeError::Unavailable) => Vec::new(),
        Err(ProbeError::Failed(reason)) => {
            tracing::warn!("dkms status: {reason}");
            failure(out, "dkms could not list its modules", &reason);
            return Exit::Unavailable;
        }
    };

    match command {
        DkmsCommand::List(list) => print_all(&list, &module_rows(&entries), out, module_listing),
        DkmsCommand::Build(args) => {
            let snapshot = system();
            let planned = build(&args, &entries, &snapshot);
            run(planned, args.confirm, None, runtime, out)
        }
    }
}

/// One setting of the page, as the user would pick it there.
///
/// # Errors
///
/// [`Refused`] for a value the setting does not take, or what the page
/// refuses.
pub fn set(args: &BootSet, snapshot: &Snapshot) -> Result<Plan, Refused> {
    boot_edit(setting(args, snapshot)?, snapshot)
}

/// The page's edit behind `args`.
///
/// # Errors
///
/// [`Refused::Rule`] for a value the setting does not take.
pub fn setting(args: &BootSet, snapshot: &Snapshot) -> Result<Edit, Refused> {
    let value = args.value.as_str();
    let change = match args.key {
        BootKey::Default => Edit::Default(value.to_owned()),
        BootKey::Timeout => Edit::Timeout(value.parse().map_err(|_nan| {
            Refused::Rule(format!("`{value}` is no number of seconds, 0 to 300"))
        })?),
        BootKey::Remember => Edit::Remember {
            on: on_off(value)?,
            fallback: shown(snapshot, &ChangeSet::default())
                .map(|now| now.default)
                .unwrap_or_default(),
        },
        BootKey::Submenus => Edit::Submenus(on_off(value)?),
        BootKey::OsProber => Edit::OsProber(on_off(value)?),
        BootKey::Quiet => Edit::Flag(Flag::Quiet, on_off(value)?),
        BootKey::Splash => Edit::Flag(Flag::Splash, on_off(value)?),
        BootKey::MitigationsOff => Edit::Flag(Flag::MitigationsOff, on_off(value)?),
        BootKey::Sysrq => Edit::Flag(Flag::SysRq, on_off(value)?),
        BootKey::Background => Edit::Background(path_or_none(value)),
        BootKey::Theme => Edit::Theme(path_or_none(value)),
        BootKey::Plymouth => Edit::PlymouthTheme(value.to_owned()),
        BootKey::Editor | BootKey::AutoEntries | BootKey::AutoFirmware => {
            return Err(Refused::Rule(
                "this is a systemd-boot setting, this system starts with GRUB".to_owned(),
            ));
        }
    };

    Ok(change)
}

/// The default entry set to `kernel`'s entry in `menu`.
///
/// # Errors
///
/// [`Refused`] for a kernel the menu has no entry of.
pub fn default_kernel(
    kernel: &PackageName,
    menu: &GrubMenu,
    snapshot: &Snapshot,
) -> Result<Plan, Refused> {
    let Some(entry) = menu.kernel_entry(kernel) else {
        let known = kernel_names(snapshot);
        let text = format!("the boot menu has no entry of `{kernel}`, see `boot entries`");
        return Err(
            Refused::Unknown(text).hinted(kernel.as_str(), known.iter().map(String::as_str))
        );
    };

    boot_edit(Edit::Default(entry.id.as_str().to_owned()), snapshot)
}

/// The builds every installed kernel with headers lacks, or only those of
/// `args.kernel`.
///
/// # Errors
///
/// [`Refused::NothingToDo`] when nothing lacks a build, [`Refused`] for
/// a kernel that is not installed.
pub fn build(
    args: &DkmsBuild,
    entries: &[DkmsEntry],
    snapshot: &Snapshot,
) -> Result<Plan, Refused> {
    fresh(|changes| build_onto(args, entries, snapshot, changes))
}

/// [`build`] onto `changes`.
///
/// # Errors
///
/// As [`build`].
pub fn build_onto(
    args: &DkmsBuild,
    entries: &[DkmsEntry],
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let releases = snapshot
        .part::<KernelReleases>()
        .cloned()
        .unwrap_or_default();

    let packages = snapshot
        .part::<LinuxPackages>()
        .ok_or_else(|| Refused::Rule("pacman's databases could not be read".to_owned()))?;

    let kernels = list_kernels(packages, &releases);

    if let Some(wanted) = &args.kernel
        && !kernels.iter().any(|kernel| kernel.name == *wanted)
    {
        let known = kernels.iter().map(|kernel| kernel.name.as_str());
        let text = format!("`{wanted}` is no kernel, see `kernels list`");
        return Err(Refused::Unknown(text).hinted(wanted.as_str(), known));
    }

    let missing: Vec<Build> = kernels
        .iter()
        .filter(|kernel| {
            args.kernel
                .as_ref()
                .is_none_or(|wanted| kernel.name == *wanted)
        })
        .filter(|kernel| kernel.is_installed() && kernel.headers_installed)
        .filter_map(|kernel| releases.0.get(&kernel.name))
        .flat_map(|release| builds(release, entries))
        .collect();

    if missing.is_empty() {
        return Err(Refused::NothingToDo(
            "Nothing to do: every module is built for its kernels.".to_owned(),
        ));
    }

    for one in missing {
        changes.want::<DkmsBuilds>(one, Queued, snapshot);
    }

    Ok(())
}

fn module_rows(entries: &[DkmsEntry]) -> Vec<ModuleRow> {
    entries
        .iter()
        .map(|entry| ModuleRow {
            module: entry.module.as_str().to_owned(),
            version: entry.version.as_str().to_owned(),
            kernel: entry
                .kernel
                .as_ref()
                .map(|release| release.as_str().to_owned())
                .unwrap_or_default(),
            state: match entry.state {
                DkmsState::Added => "added",
                DkmsState::Built => "built",
                DkmsState::Installed => "installed",
                DkmsState::Broken => "broken",
            },
        })
        .collect()
}

fn boot_edit(change: Edit, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| edit_onto(change, snapshot, changes))
}

/// One edit of the page onto `changes`.
///
/// # Errors
///
/// [`Refused::Rule`] with the English line of the page's refusal.
pub fn edit_onto(
    change: Edit,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let value = change.value();

    page_edit(
        changes,
        snapshot,
        |problem: EditProblem| why::boot_edit(problem, &value),
        |changes, snapshot| edit(change, changes, snapshot),
    )
}

fn show(read: Read, snapshot: &Snapshot, out: &mut dyn Write) -> Exit {
    let Some(now) = shown(snapshot, &ChangeSet::default()) else {
        return unavailable("GRUB is not the boot loader here", out);
    };

    let view = BootView {
        default: now.default,
        remember: now.remember,
        timeout: now.timeout,
        parameters: now
            .parameters
            .map(|parameters| parameters.iter().map(ToString::to_string).collect()),
        submenus: now.submenus,
        os_prober: now.os_prober,
        background: now.background,
        theme: now.theme,
    };

    let text = if read.json {
        json(&view)
    } else {
        let timeout = view
            .timeout
            .map_or_else(String::new, |seconds| seconds.to_string());

        let parameters = view
            .parameters
            .as_ref()
            .map_or_else(String::new, |list| list.join(" "));

        let paint = crate::paint();
        let facts = paint.facts(
            "Boot menu",
            &[
                ("default", &view.default),
                ("remember", on_off_text(view.remember)),
                ("timeout", &timeout),
                ("parameters", &parameters),
                ("submenus", on_off_text(view.submenus)),
                ("os-prober", on_off_text(view.os_prober)),
                ("background", &view.background),
                ("theme", &view.theme),
            ],
        );

        format!(
            "{facts}\n{}\n",
            paint.hint("garuda-toolbox boot set <setting> <value>")
        )
    };

    write!(out, "{text}").ok();
    Exit::Success
}

fn entries(read: Read, out: &mut dyn Write) -> Exit {
    match read_menu() {
        Some(menu) => print(
            read,
            &toolbox_core::features::boot::view::entries(Some(&menu)),
            out,
            entry_listing,
        ),
        None => unavailable("the boot menu could not be read", out),
    }
}

fn read_menu() -> Option<GrubMenu> {
    let source = SystemBootMenu::system(Arc::new(super::root::helper()));
    match source.menu(&Arc::new(AtomicBool::new(false))) {
        MenuRead::Known(menu) => Some(menu),
        other @ (MenuRead::SdBoot(_)
        | MenuRead::Missing
        | MenuRead::Refused(_)
        | MenuRead::Unknown(_)
        | MenuRead::Cancelled) => {
            tracing::warn!("the boot menu was not read: {other:?}");
            None
        }
    }
}

fn on_off(value: &str) -> Result<bool, Refused> {
    super::settings::on_off(value).map_err(Refused::Rule)
}

fn path_or_none(value: &str) -> String {
    if value == "none" {
        String::new()
    } else {
        value.to_owned()
    }
}
