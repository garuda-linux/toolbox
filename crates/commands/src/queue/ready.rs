//! Around `queue apply`: no probe reads the lock flags, the AUR's names
//! or the loader's settings. The command line reads them here once a
//! restored change waits for them. A guard that still waits then fails
//! closed.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use toolbox_aur::AurSource;
use toolbox_cli::Exit;
use toolbox_core::domains::{AccountLockFlags, CurrentLoader};
use toolbox_core::features::packages::aur::{AurCatalog, AurNames};
use toolbox_core::features::{Guard, Refusal, Unchecked};
use toolbox_core::{ChangeSet, SavedChanges, Snapshot};
use toolbox_services::unprobed::{Needs, UnprobedReads, joined};
use toolbox_system::{HelperLocks, LockSource as _};

use super::said::say;
use crate::out::{Out, Settings};

/// The helper's answers and the AUR's copy.
pub struct HelperReads<'a> {
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
/// waits for them.
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

/// `probed` after the run of `changes`, with the parts their domains
/// compare with, so what the run met leaves the queue.
pub fn after(probed: Snapshot, changes: &ChangeSet, reads: &dyn UnprobedReads) -> Snapshot {
    joined(&probed, Needs::of(changes, []), reads).unwrap_or(probed)
}

pub fn refused(refusal: Refusal, out: &mut Out<'_>) -> Exit {
    let Refusal::GuardsWaiting { guard } = refusal;
    let (error, hint) = match guard {
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
    };

    say(out, error, hint);
    Exit::Unavailable
}
