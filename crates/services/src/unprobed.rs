//! The parts of a snapshot no probe reads.
//!
//! Guards check the lock flags and the AUR's names, a loader change
//! compares with the systemd-boot settings. The window reads them for its
//! pages, the command line on demand, and both join them here so a queue
//! settles the same way in each.

#[cfg(test)]
mod tests;

use toolbox_core::domains::{AccountLockFlags, CurrentLoader, LoaderSettingsDomain};
use toolbox_core::features::Guard;
use toolbox_core::features::packages::aur::AurNames;
use toolbox_core::{ChangeSet, Snapshot};

/// Each read is `None` while its part is unknown.
pub trait UnprobedReads {
    fn locks(&self) -> Option<AccountLockFlags>;

    fn aur_names(&self) -> Option<AurNames>;

    fn loader(&self) -> Option<CurrentLoader>;
}

/// Which parts a queue waits for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Needs {
    pub locks: bool,
    pub aur: bool,
    pub loader: bool,
}

impl Needs {
    /// For a reader whose parts cost nothing, such as the window's cached
    /// page reads.
    pub const ALL: Self = Self {
        locks: true,
        aur: true,
        loader: true,
    };

    /// The parts the `holding` guards wait for, and the loader's settings
    /// while a loader change is queued, as no probe ever shows it met.
    pub fn of(changes: &ChangeSet, holding: impl IntoIterator<Item = Guard>) -> Self {
        let mut needs = Self {
            loader: !changes.queued::<LoaderSettingsDomain>().is_empty(),
            ..Self::default()
        };

        for guard in holding {
            needs.locks |= guard == Guard::UserLocks;
            needs.aur |= guard == Guard::AurPackages;
        }

        needs
    }
}

/// `probed` with each needed part `reads` knows, or `None` when it added
/// none, so the caller keeps the snapshot it has.
pub fn joined(probed: &Snapshot, needs: Needs, reads: &dyn UnprobedReads) -> Option<Snapshot> {
    let locks = needs.locks.then(|| reads.locks()).flatten();
    let aur = needs.aur.then(|| reads.aur_names()).flatten();
    let loader = needs.loader.then(|| reads.loader()).flatten();

    if locks.is_none() && aur.is_none() && loader.is_none() {
        return None;
    }

    let mut snapshot = probed.clone();
    if let Some(flags) = locks {
        snapshot.insert(flags);
    }

    if let Some(names) = aur {
        snapshot.insert(names);
    }

    if let Some(loader) = loader {
        snapshot.insert(loader);
    }

    Some(snapshot)
}
