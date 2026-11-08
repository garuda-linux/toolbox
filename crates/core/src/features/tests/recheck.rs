//! A restored queue meets each page's guards once, when the snapshot
//! holds what that page checks against.

use std::collections::BTreeSet;

use toolbox_plan::{GrubState, HomeFolder};
use toolbox_state::probes::FailedProbes;

use crate::domains::{
    AccountLockFlags, AccountLocks, Accounts, AurPackages, CurrentGrub, CurrentSystemLocale,
    CurrentUser, EnabledLocales, InstalledPackages, PlymouthState, RemovedUsers, SyncPackages,
};
use crate::features::kernels::LinuxPackages;
use crate::features::network::ModuleFacts;
use crate::features::packages::aur::AurNames;
use crate::features::{GUARDS, Unchecked};
use crate::testing::{package, user_name};
use crate::{ChangeSet, Queued, Snapshot, Toggle};

/// A removal no guard lets through: nobody of that name exists.
fn removal(changes: &mut ChangeSet) {
    changes.want::<RemovedUsers>(user_name("ghost"), HomeFolder::Keep, &Snapshot::default());
}

/// What every guard checks against, so each of them runs.
fn every_part() -> Snapshot {
    let mut snapshot = without_aur();
    snapshot.insert(AurNames::default());
    snapshot
}

/// Every part but the AUR's copy, which the window reads only on demand.
fn without_aur() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Accounts::default());
    snapshot.insert(CurrentUser(user_name("nico")));
    snapshot.insert(AccountLockFlags::default());
    snapshot.insert(EnabledLocales(BTreeSet::new()));
    snapshot.insert(CurrentSystemLocale(None));
    snapshot.insert(PlymouthState {
        themes: Vec::new(),
        current: None,
    });
    snapshot.insert(CurrentGrub(GrubState::read("")));
    snapshot.insert(ModuleFacts::default());
    snapshot.insert(FailedProbes::default());
    snapshot.insert(SyncPackages::default());
    snapshot.insert(InstalledPackages::default());
    snapshot
}

#[test]
fn a_guard_waits_for_the_part_it_checks_against() {
    let mut unchecked = Unchecked::all();
    let mut changes = ChangeSet::default();
    let mut kernels_only = Snapshot::default();

    kernels_only.insert(LinuxPackages::default());
    removal(&mut changes);

    assert_eq!(unchecked.recheck(&mut changes, &Snapshot::default()), 0);
    assert_eq!(unchecked, Unchecked::all());
    assert_eq!(unchecked.recheck(&mut changes, &kernels_only), 0);
    assert_eq!(changes.len(), 1, "the accounts are not known yet");

    assert_eq!(unchecked.recheck(&mut changes, &every_part()), 1);
    assert!(changes.is_empty());
    assert_eq!(unchecked, Unchecked::default());
}

#[test]
fn a_guard_runs_once() {
    let mut unchecked = Unchecked::all();
    let mut changes = ChangeSet::default();

    assert_eq!(unchecked.recheck(&mut changes, &every_part()), 0);
    removal(&mut changes);

    assert_eq!(unchecked.recheck(&mut changes, &every_part()), 0);
    assert_eq!(changes.len(), 1, "what came later met the page's guard");
    assert_ne!(unchecked, Unchecked::default(), "the kernels still wait");
}

#[test]
fn nothing_restored_is_nothing_to_check() {
    let mut unchecked = Unchecked::default();
    let mut changes = ChangeSet::default();

    removal(&mut changes);

    assert_eq!(unchecked.recheck(&mut changes, &every_part()), 0);
    assert_eq!(changes.len(), 1);
}

/// The lock flags come from the helper when the Users page asks, long
/// after the account files. Restored locks wait for them, the rest of the
/// account changes do not.
#[test]
fn restored_locks_wait_for_the_lock_flags() {
    let mut unchecked = Unchecked::all();
    let mut changes = ChangeSet::default();
    let mut without_flags = Snapshot::default();

    without_flags.insert(Accounts::default());
    without_flags.insert(CurrentUser(user_name("nico")));
    removal(&mut changes);
    changes.want::<AccountLocks>(user_name("ghost"), Toggle::On, &Snapshot::default());

    assert_eq!(unchecked.recheck(&mut changes, &without_flags), 1);
    assert_eq!(changes.len(), 1, "the lock waits for the flags");

    assert_eq!(unchecked.recheck(&mut changes, &every_part()), 1);
    assert!(changes.is_empty());
}

#[test]
fn every_guard_has_an_id_of_its_own() {
    let ids: BTreeSet<&str> = GUARDS.iter().map(|(id, _)| *id).collect();

    assert_eq!(ids.len(), GUARDS.len());
}

/// An AUR build from the bus or a file waits for the AUR's copy, then
/// goes when the copy has no package of that name.
#[test]
fn queued_aur_builds_wait_for_the_aur_copy_and_unknown_names_go() {
    let mut unchecked = Unchecked::all();
    let mut changes = ChangeSet::default();
    let empty = Snapshot::default();
    changes.want::<AurPackages>(package("yay-bin"), Queued, &empty);
    changes.want::<AurPackages>(package("not-in-aur"), Queued, &empty);

    assert_eq!(unchecked.recheck(&mut changes, &without_aur()), 0);
    assert_eq!(changes.len(), 2, "the builds wait for the copy");

    let mut with_copy = without_aur();
    with_copy.insert(AurNames(std::iter::once(package("yay-bin")).collect()));
    assert_eq!(unchecked.recheck(&mut changes, &with_copy), 1);
    assert_eq!(
        changes.queued::<AurPackages>(),
        [(package("yay-bin"), Queued)]
    );
}
