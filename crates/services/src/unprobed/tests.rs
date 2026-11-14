use std::cell::Cell;

use toolbox_core::domains::{AccountLockFlags, CurrentLoader, LoaderSettingsDomain};
use toolbox_core::features::Guard;
use toolbox_core::features::packages::aur::AurNames;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::sdboot::{LoaderSetting, LoaderSettings, LoaderTimeout};

use super::{Needs, UnprobedReads, joined};

/// Knows every part and counts the reads, as one may cost a helper call.
#[derive(Default)]
struct Known {
    reads: Cell<usize>,
}

impl Known {
    fn read<T>(&self, part: T) -> T {
        self.reads.set(self.reads.get() + 1);
        part
    }
}

impl UnprobedReads for Known {
    fn locks(&self) -> Option<AccountLockFlags> {
        Some(self.read(AccountLockFlags::default()))
    }

    fn aur_names(&self) -> Option<AurNames> {
        Some(self.read(AurNames(["paru".parse().unwrap()].into())))
    }

    fn loader(&self) -> Option<CurrentLoader> {
        Some(self.read(CurrentLoader(LoaderSettings::default())))
    }
}

/// Knows nothing, as when the helper does not answer.
struct Unknown;

impl UnprobedReads for Unknown {
    fn locks(&self) -> Option<AccountLockFlags> {
        None
    }

    fn aur_names(&self) -> Option<AurNames> {
        None
    }

    fn loader(&self) -> Option<CurrentLoader> {
        None
    }
}

fn loader_change() -> ChangeSet {
    let mut changes = ChangeSet::default();
    let setting = LoaderSetting::Timeout(Some(LoaderTimeout::Seconds(5)));
    LoaderSettingsDomain::want(&mut changes, setting, &Snapshot::default());
    changes
}

#[test]
fn an_empty_queue_that_no_guard_holds_needs_nothing() {
    let needs = Needs::of(&ChangeSet::default(), []);

    assert_eq!(needs, Needs::default());
    assert!(!needs.any());
}

#[test]
fn a_held_guard_needs_the_part_it_checks() {
    assert_eq!(
        Needs::of(&ChangeSet::default(), [Guard::Users, Guard::UserLocks]),
        Needs {
            locks: true,
            ..Needs::default()
        }
    );

    assert_eq!(
        Needs::of(&ChangeSet::default(), [Guard::AurPackages]),
        Needs {
            aur: true,
            ..Needs::default()
        }
    );
}

/// No probe reads loader.conf, so without the read a loader change that
/// applied stays queued and runs again.
#[test]
fn a_queued_loader_change_needs_the_loader_without_a_guard() {
    let needs = Needs::of(&loader_change(), []);

    assert_eq!(
        needs,
        Needs {
            loader: true,
            ..Needs::default()
        }
    );
}

#[test]
fn only_the_needed_parts_are_read_and_joined() {
    let reads = Known::default();
    let needs = Needs {
        loader: true,
        ..Needs::default()
    };

    let snapshot = joined(&Snapshot::default(), needs, &reads).unwrap();

    assert_eq!(reads.reads.get(), 1);
    assert!(snapshot.part::<CurrentLoader>().is_some());
    assert!(snapshot.part::<AccountLockFlags>().is_none());
    assert!(snapshot.part::<AurNames>().is_none());
}

#[test]
fn every_part_joins_when_all_are_needed() {
    let snapshot = joined(&Snapshot::default(), Needs::ALL, &Known::default()).unwrap();

    assert!(snapshot.part::<CurrentLoader>().is_some());
    assert!(snapshot.part::<AccountLockFlags>().is_some());
    assert!(snapshot.part::<AurNames>().is_some());
}

/// The caller keeps the snapshot it has, so a page that compares by
/// pointer sees no change.
#[test]
fn nothing_read_joins_nothing() {
    assert!(joined(&Snapshot::default(), Needs::ALL, &Unknown).is_none());
    assert!(joined(&Snapshot::default(), Needs::default(), &Known::default()).is_none());
}
