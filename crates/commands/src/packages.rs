//! The Packages page's commands: a search over the repository and the
//! installed packages or the AUR, one package shown, and an install or a
//! removal through the page's own rule.

mod aur;
mod rows;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::io::Write as _;

pub use aur::{AurInfo, AurRow, aur_info_text, aur_listing};
pub use rows::{PackageInfo, PackageRow, info_text, listing};
use tokio::runtime::Runtime;
use toolbox_aur::{AurError, AurSource, Cache};
use toolbox_cli::Exit;
use toolbox_cli::args::{
    PackageArgs, PackageInstall, PackageShow, PackagesCommand, PackagesSearch,
};
use toolbox_cli::output::{Paint, json};
use toolbox_config::Paths;
use toolbox_core::domains::{InstalledPackages, SyncPackages};
use toolbox_core::features::packages::list::{Catalog, Filter};
use toolbox_core::features::packages::queue::{Refusal, toggle};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::{PackageName, Plan, Protection};
use toolbox_system::{Libalpm, PackageIndexSource, PackageSource as _, ProbeError};

use super::act::run;
use super::change::{Refused, fresh};
use super::out::{failure, print};
use super::{spinner, system, unavailable};
use crate::out::Out;

pub fn command(command: PackagesCommand, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    match command {
        PackagesCommand::Search(args) if args.aur => search_aur(&args, &aur_copy(), out),
        PackagesCommand::Search(args) => search(&args, &Libalpm::system(), out),
        PackagesCommand::Show(args) if args.aur => show_aur(&args, &aur_copy(), out),
        PackagesCommand::Show(args) => show(&args, &Libalpm::system(), out),
        PackagesCommand::Install(args) => run(
            install(&args, &aur_copy(), &system()),
            args.confirm,
            None,
            runtime,
            out,
        ),
        PackagesCommand::Remove(args) => run(
            change(&args, Toggle::Off, &system()),
            args.confirm,
            None,
            runtime,
            out,
        ),
    }
}

/// The rows a search keeps: every package, the installed ones or the
/// orphans, whose name or description holds the text.
///
/// # Errors
///
/// What the index read gave.
pub fn found(
    args: &PackagesSearch,
    source: &dyn PackageIndexSource,
) -> Result<Vec<PackageRow>, ProbeError> {
    let catalog = Catalog::new(source.index()?);
    let filter = if args.orphans {
        Filter::Orphans
    } else if args.installed {
        Filter::Installed
    } else {
        Filter::All
    };

    let view = catalog.view(
        filter,
        args.text.as_deref().unwrap_or_default(),
        &BTreeSet::default(),
    );

    let entries = catalog.entries();

    Ok(view
        .rows
        .iter()
        .filter_map(|&place| entries.get(place))
        .map(PackageRow::of)
        .collect())
}

fn search(args: &PackagesSearch, source: &dyn PackageIndexSource, out: &mut Out<'_>) -> Exit {
    match spinner::while_reading(args.read.json || out.quiet(), || found(args, source)) {
        Ok(rows) => print(args.read, &rows, out, listing),
        Err(error) => unreadable(&error, out),
    }
}

fn show(args: &PackageShow, source: &dyn PackageIndexSource, out: &mut Out<'_>) -> Exit {
    let read = spinner::while_reading(args.read.json || out.quiet(), || {
        source.details(&args.package)
    });

    let details = match read {
        Ok(Some(details)) => details,
        Ok(None) => {
            super::out::say_error(
                out,
                &format!(
                    "`{}` is in no package database, see `packages search`",
                    args.package
                ),
            );

            return Exit::Usage;
        }
        Err(error) => return unreadable(&error, out),
    };

    let info = PackageInfo::of(details);
    if args.read.json {
        write!(out, "{}", json(&info)).ok();
    } else {
        write!(out, "{}", info_text(out.paint(), &info)).ok();
    }

    Exit::Success
}

/// The AUR's copy in the cache folder, downloaded when a day old. The
/// `--aur` flag is the ask, so this runs whatever the window's setting.
fn aur_copy() -> Cache {
    Cache::system(Paths::from_process().aur_dir())
}

/// The installed packages of the local database. Without pacman nothing
/// is installed through it.
fn installed() -> Result<BTreeSet<PackageName>, ProbeError> {
    match Libalpm::system().installed() {
        Ok(installed) => Ok(installed.0),
        Err(ProbeError::Unavailable) => Ok(BTreeSet::new()),
        Err(error) => Err(error),
    }
}

fn listed(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
    }
}

fn facts_text(paint: Paint, title: &str, facts: &[(&str, String)]) -> String {
    let pairs: Vec<(&str, &str)> = facts
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();

    paint.facts(title, &pairs)
}

fn search_aur(args: &PackagesSearch, aur: &dyn AurSource, out: &mut Out<'_>) -> Exit {
    let installed = match installed() {
        Ok(installed) => installed,
        Err(error) => return unreadable(&error, out),
    };

    let read = spinner::while_reading(args.read.json || out.quiet(), || {
        aur::found(args.text.as_deref(), aur, &installed)
    });

    match read {
        Ok(rows) => print(args.read, &rows, out, aur_listing),
        Err(error) => aur_unreadable(&error, out),
    }
}

fn show_aur(args: &PackageShow, aur: &dyn AurSource, out: &mut Out<'_>) -> Exit {
    let installed = match installed() {
        Ok(installed) => installed,
        Err(error) => return unreadable(&error, out),
    };

    let read = spinner::while_reading(args.read.json || out.quiet(), || {
        aur::aur_info(&args.package, aur, &installed)
    });

    let info = match read {
        Ok(Some(info)) => info,
        Ok(None) => {
            super::out::say_error(
                out,
                &format!(
                    "`{}` is not in the AUR, see `packages search --aur`",
                    args.package
                ),
            );
            return Exit::Usage;
        }
        Err(error) => return aur_unreadable(&error, out),
    };

    if args.read.json {
        write!(out, "{}", json(&info)).ok();
    } else {
        write!(out, "{}", aur_info_text(out.paint(), &info)).ok();
    }

    Exit::Success
}

fn aur_unreadable(error: &AurError, out: &mut Out<'_>) -> Exit {
    match error {
        AurError::Waiting { next } => unavailable(&aur::waiting_text(*next), out),
        AurError::Download(reason) | AurError::Storage(reason) => {
            failure(out, "the AUR's package list could not be read", reason);
            Exit::Failure
        }
    }
}

fn unreadable(error: &ProbeError, out: &mut Out<'_>) -> Exit {
    match error {
        ProbeError::Unavailable => unavailable("pacman's databases could not be read", out),
        ProbeError::Failed(_) | ProbeError::Io { what: _, error: _ } => {
            let reason = error.reason().unwrap_or_default();
            failure(out, "pacman's databases could not be read", &reason);
            Exit::Failure
        }
    }
}

/// An install of the repository packages, or the AUR builds with
/// `--aur`, as the page queues them.
///
/// # Errors
///
/// As [`change_onto`] and [`aur::install_onto`].
pub fn install(
    args: &PackageInstall,
    aur: &dyn AurSource,
    snapshot: &Snapshot,
) -> Result<Plan, Refused> {
    fresh(|changes| install_onto(args, aur, snapshot, changes))
}

/// The wishes of [`install`] added to a queue, for `queue add`.
///
/// # Errors
///
/// As [`install`].
pub fn install_onto(
    args: &PackageInstall,
    aur: &dyn AurSource,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    if args.aur {
        return aur::install_onto(&args.packages, aur, snapshot, changes);
    }

    args.packages
        .iter()
        .try_for_each(|package| wish_onto(package, Toggle::On, snapshot, changes))
}

/// An install or a removal of one package, as the page queues it.
///
/// # Errors
///
/// [`Refused`] for a package no repository offers, one installed or
/// missing already, or a system whose packages could not be read.
pub fn change(args: &PackageArgs, want: Toggle, snapshot: &Snapshot) -> Result<Plan, Refused> {
    fresh(|changes| change_onto(args, want, snapshot, changes))
}

/// The wish of [`change`] added to a queue, for `queue add`.
///
/// # Errors
///
/// As [`change`].
pub fn change_onto(
    args: &PackageArgs,
    want: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    wish_onto(&args.package, want, snapshot, changes)
}

fn wish_onto(
    package: &PackageName,
    want: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    let installed = snapshot
        .part::<InstalledPackages>()
        .is_some_and(|installed| installed.0.contains(package));

    if installed == want.is_on() {
        let state = if installed {
            "installed"
        } else {
            "not installed"
        };

        return Err(Refused::NothingToDo(format!(
            "`{package}` is {state} already"
        )));
    }

    toggle(package, want, snapshot, changes).map_err(|refusal| match refusal {
        Refusal::Unread => Refused::Unavailable("pacman's databases could not be read".to_owned()),
        Refusal::NotOffered => Refused::Unknown(format!(
            "`{package}` is in no configured repository, see `packages search`"
        ))
        .hinted(
            package.as_str(),
            snapshot
                .part::<SyncPackages>()
                .iter()
                .flat_map(|offered| offered.0.iter().map(PackageName::as_str)),
        ),
        kept @ (Refusal::RunningKernel
        | Refusal::RunningUnknown
        | Refusal::LastKernel
        | Refusal::Protected(_)
        | Refusal::TakesAlong(..)) => kept_package(package, kept),
    })
}

/// The protected list and the Kernels page's rules, for a removal asked
/// on another page.
pub(crate) fn kept_package(package: &PackageName, refusal: Refusal) -> Refused {
    Refused::Rule(match refusal {
        Refusal::Protected(why) => format!("{package} stays installed: {}", protection(why)),
        Refusal::TakesAlong(why, along) => format!(
            "{package} stays installed: removing it would also remove {along}, and {}",
            protection(why)
        ),
        Refusal::RunningKernel => format!(
            "{package} runs right now, so it stays. Restart with another kernel to remove it."
        ),
        Refusal::LastKernel => {
            format!("{package} is the last installed kernel. Install another one first.")
        }
        Refusal::RunningUnknown | Refusal::Unread | Refusal::NotOffered => format!(
            "{package} stays: the kernel that runs right now is not known. Restart and try again."
        ),
    })
}

const fn protection(why: Protection) -> &'static str {
    match why {
        Protection::Boot => "the system needs it to start",
        Protection::Root => "it gives administrator rights",
        Protection::Login => "it is needed to log in",
        Protection::Network => "it keeps the network connection",
        Protection::PackageManager => "it installs and updates software",
        Protection::Desktop => "the desktop needs it",
    }
}
