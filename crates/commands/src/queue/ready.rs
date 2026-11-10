//! Before `queue apply`: no probe reads the lock flags or the AUR's
//! names. The window reads them for its pages, the command line reads
//! them here once a restored change waits for them. A guard that still
//! waits then fails closed.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use toolbox_aur::AurSource;
use toolbox_cli::Exit;
use toolbox_cli::output::ColorMode;
use toolbox_core::domains::AccountLockFlags;
use toolbox_core::features::packages::aur::{AurCatalog, AurNames};
use toolbox_core::features::{Guard, Refusal, Unchecked};
use toolbox_core::{ChangeSet, SavedChanges, Snapshot};
use toolbox_system::{HelperLocks, LockSource as _};

use super::said::say;
use crate::out::Out;

/// The parts of a snapshot only a read outside the probe gives.
pub trait PageReads {
    /// `None` when the helper gave no flags.
    fn locks(&self) -> Option<AccountLockFlags>;

    /// `None` when the AUR's copy cannot be read.
    fn aur_names(&self) -> Option<AurNames>;
}

pub struct SystemReads<'a> {
    /// For the note when the helper goes through sudo.
    pub color: ColorMode,
    pub aur: &'a dyn AurSource,
}

impl PageReads for SystemReads<'_> {
    fn locks(&self) -> Option<AccountLockFlags> {
        HelperLocks(Arc::new(crate::root::helper(self.color)))
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
    reads: &dyn PageReads,
) -> Result<ChangeSet, Refusal> {
    let mut changes = super::restore(saved, system);
    let mut unchecked = Unchecked::all();
    unchecked.recheck(&mut changes, system);

    let locks = unchecked
        .holding(&changes)
        .any(|guard| guard == Guard::UserLocks);

    let aur = unchecked
        .holding(&changes)
        .any(|guard| guard == Guard::AurPackages);

    if locks || aur {
        let mut snapshot = system.clone();
        if locks && let Some(flags) = reads.locks() {
            snapshot.insert(flags);
        }

        if aur && let Some(names) = reads.aur_names() {
            snapshot.insert(names);
        }

        unchecked.recheck(&mut changes, &snapshot);
    }

    unchecked.ready(&changes)?;
    Ok(changes)
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
