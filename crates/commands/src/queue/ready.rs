//! Around `queue apply`: no probe reads the lock flags, the AUR's names
//! or the loader's settings. The command line reads them here once a
//! restored change waits for them. A guard that still waits then fails
//! closed.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use toolbox_aur::AurSource;
use toolbox_cli::Exit;
use toolbox_core::domains::{AccountLockFlags, CurrentLoader, waiting_orders};
use toolbox_core::features::packages::aur::{AurCatalog, AurNames};
use toolbox_core::features::{Guard, Refusal, Unchecked};
use toolbox_core::{ChangeSet, SavedChanges, Snapshot};
use toolbox_plan::PackageName;
use toolbox_services::aur_orders::{self, RouteProblem};
use toolbox_services::unprobed::{Needs, UnprobedReads, joined};
use toolbox_system::{AurOrderSource, HelperLocks, LockSource as _};

use super::said::say;
use crate::change::Refused;
use crate::out::{Out, Settings};
use crate::packages::aur;

/// The helper's answers and the AUR's copy.
pub(super) struct HelperReads<'a> {
    /// For the note when the helper goes through sudo.
    pub settings: Settings,
    pub aur: &'a dyn AurSource,
}

impl UnprobedReads for HelperReads<'_> {
    fn locks(&self) -> Option<AccountLockFlags> {
        HelperLocks(Arc::new(crate::root::helper(self.settings)))
            .locks()
            .inspect_err(|problem| {
                tracing::warn!("the lock state of accounts is unknown: {problem:?}");
            })
            .ok()
            .map(AccountLockFlags)
    }

    fn aur_names(&self) -> Option<AurNames> {
        self.aur
            .index()
            .inspect_err(|error| tracing::warn!("{error}"))
            .ok()
            .map(|index| AurCatalog::new(index).names())
    }

    fn loader(&self) -> Option<CurrentLoader> {
        crate::boot::sdboot::read_menu(self.settings).map(|menu| CurrentLoader(menu.settings))
    }
}

/// The queue `saved` as it may run against `system`: restored and met by
/// every guard, with the parts of `reads` read only for a change that
/// waits for them. The build orders are not read here, see [`filled`].
///
/// # Errors
///
/// [`Refusal::GuardsWaiting`] when a guard still waits for its part. Its
/// changes are unchecked then, so none of the queue runs.
pub fn checked(
    saved: &SavedChanges,
    system: &Snapshot,
    reads: &dyn UnprobedReads,
) -> Result<ChangeSet, Refusal> {
    let mut changes = super::restore(saved, system);
    let mut unchecked = Unchecked::all();
    unchecked.recheck(&mut changes, system);

    // No guard checks the loader. `after` reads it once the run changed it.
    let needs = Needs {
        loader: false,
        ..Needs::of(&changes, unchecked.holding(&changes))
    };

    if let Some(snapshot) = joined(system, needs, reads) {
        unchecked.recheck(&mut changes, &snapshot);
    }

    unchecked.ready(&changes)?;
    Ok(changes)
}

/// Refuses an AUR build that waits for the window, when the command line
/// starts the helper through sudo. Sudo does not say who asked, so the
/// build's helper step could not find the build folder.
///
/// # Errors
///
/// [`Refused::Unavailable`] naming the waiting builds.
fn sudo_waits(changes: &ChangeSet, through_sudo: bool) -> Result<(), Refused> {
    let waiting = aur_orders::needing(changes);
    if !through_sudo || waiting.is_empty() {
        return Ok(());
    }

    Err(Refused::Unavailable(format!(
        "{} need the window or a polkit agent, sudo cannot start their build\napply the queue in the window, or run this where pkexec can ask",
        names(&waiting)
    )))
}

/// The checks an apply makes before its plan: a sudo run takes no waiting
/// build, and every build's order is read and filled.
///
/// # Errors
///
/// As [`sudo_waits`] and [`filled`].
pub(super) fn prepared(
    changes: &mut ChangeSet,
    system: &Snapshot,
    through_sudo: bool,
    route: Result<(), RouteProblem>,
    orders: &dyn AurOrderSource,
) -> Result<(), Refused> {
    sudo_waits(changes, through_sudo)?;
    filled(changes, system, route, orders)
}

/// Reads and fills the build orders of the queued AUR builds before the
/// plan, as the window does before every Apply. Nothing builds on a
/// queue that is not all built.
///
/// # Errors
///
/// [`Refused::Unavailable`] naming the first build without its order, or
/// [`Refusal::OrderWaits`] for a build left waiting.
pub fn filled(
    changes: &mut ChangeSet,
    system: &Snapshot,
    route: Result<(), RouteProblem>,
    orders: &dyn AurOrderSource,
) -> Result<(), Refused> {
    aur::filled(changes, system, orders, route, crate::root::run_counter())?;

    let left: Vec<PackageName> = waiting_orders(changes);
    if left.is_empty() {
        Ok(())
    } else {
        Err(Refused::Unavailable(order_words(&left)))
    }
}

fn names(packages: &[PackageName]) -> String {
    packages
        .iter()
        .map(PackageName::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

/// `probed` after the run of `changes`, with the parts their domains
/// compare with, so what the run met leaves the queue.
pub fn after(probed: Snapshot, changes: &ChangeSet, reads: &dyn UnprobedReads) -> Snapshot {
    joined(&probed, Needs::of(changes, []), reads).unwrap_or(probed)
}

pub fn refused(refusal: Refusal, out: &mut Out<'_>) -> Exit {
    match refusal {
        Refusal::GuardsWaiting { guard } => {
            let (error, hint) = guard_words(guard);
            say(out, error, hint);
        }
        Refusal::OrderWaits { names } => say(
            out,
            &order_words(&names),
            "take the AUR builds out of the queue with garuda-toolbox queue remove, then apply the rest",
        ),
    }

    Exit::Unavailable
}

fn order_words(packages: &[PackageName]) -> String {
    let list = names(packages);

    if packages.len() == 1 {
        format!("{list} waits for its build order")
    } else {
        format!("{list} wait for their build orders")
    }
}

const fn guard_words(guard: Guard) -> (&'static str, &'static str) {
    match guard {
        Guard::UserLocks => (
            "a queued account change waits for the lock state of the accounts",
            "the helper did not answer: try again, or take the change back with garuda-toolbox queue remove",
        ),
        Guard::AurPackages => (
            "a queued AUR build waits for the package list of the AUR",
            "the list could not be read: try again later, or take the build back with garuda-toolbox queue remove",
        ),
        Guard::Users
        | Guard::Memberships
        | Guard::Kernels
        | Guard::Locales
        | Guard::LanguagePacks
        | Guard::PackageRemovals
        | Guard::Plymouth
        | Guard::Grub
        | Guard::ModuleBlacklist
        | Guard::NewGroups
        | Guard::Hblock
        | Guard::WifiBackend => (
            "a queued change waits for a part of the system that could not be read",
            "try again, or take the change back with garuda-toolbox queue remove",
        ),
    }
}
