//! A restored queue meets each page's guards once, when the snapshot
//! holds what that page checks against.

use std::collections::BTreeSet;

use toolbox_plan::{GrubState, HomeFolder};
use toolbox_state::probes::FailedProbes;

use crate::domains::{
    AccountLockFlags, AccountLocks, Accounts, AurPackages, CurrentGrub, CurrentSystemLocale,
    CurrentUser, EnabledLocales, InstalledPackages, Packages, PlymouthState, RemovedUsers,
    SyncPackages,
};
use crate::features::kernels::LinuxPackages;
use crate::features::network::ModuleFacts;
use crate::features::packages::aur::AurNames;
use crate::features::{GUARDS, Refusal, Unchecked};
use crate::test_support::{package, user_name};
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
    let mut snapshot = without_page_reads();
    snapshot.insert(AccountLockFlags::default());
    snapshot
}

/// What the probe reads, without the parts only a page's read adds.
fn without_page_reads() -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Accounts::default());
    snapshot.insert(CurrentUser(user_name("nico")));
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
    let ids: BTreeSet<&str> = GUARDS.iter().map(|guard| guard.id).collect();

    assert_eq!(ids.len(), GUARDS.len());
}

#[test]
fn every_guard_names_known_domains() {
    let known = crate::domains::all().unwrap().ids();

    for guard in GUARDS {
        for domain in guard.domains {
            assert!(known.contains(domain), "{}: {domain}", guard.id);
        }
    }
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

/// A guard that answers `None` with every part at hand would hold its
/// domains back for good.
#[test]
fn every_guard_runs_on_a_snapshot_with_every_part() {
    let mut snapshot = every_part();
    snapshot.insert(LinuxPackages::default());

    for guard in GUARDS {
        assert!(
            (guard.run)(&mut ChangeSet::default(), &snapshot).is_some(),
            "{} waits",
            guard.id
        );
    }
}

#[test]
fn a_waiting_guard_holds_back_the_plan_of_its_domains() {
    let mut unchecked = Unchecked::all();
    let mut changes = ChangeSet::default();
    let mut snapshot = without_page_reads();
    snapshot.insert(LinuxPackages::default());
    changes.want::<AccountLocks>(user_name("nico"), Toggle::On, &Snapshot::default());

    unchecked.recheck(&mut changes, &snapshot);

    assert_eq!(changes.len(), 1, "the lock waits for the flags");
    assert_eq!(
        unchecked.ready(&changes),
        Err(Refusal::GuardsWaiting {
            guard: "user-locks"
        })
    );
}

#[test]
fn a_waiting_guard_holds_back_no_other_domain() {
    let mut unchecked = Unchecked::all();
    let mut changes = ChangeSet::default();
    let mut snapshot = without_page_reads();
    snapshot.insert(LinuxPackages::default());
    changes.want::<Packages>(package("htop"), Toggle::On, &Snapshot::default());

    unchecked.recheck(&mut changes, &snapshot);

    assert_eq!(
        unchecked.waiting().collect::<Vec<_>>(),
        ["aur-packages", "user-locks"]
    );
    assert_eq!(unchecked.ready(&changes), Ok(()));
}

#[test]
fn a_queue_before_the_first_read_is_held_back() {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(package("htop"), Toggle::On, &Snapshot::default());

    assert!(Unchecked::all().ready(&changes).is_err());
    assert_eq!(Unchecked::default().ready(&changes), Ok(()));
}

/// Both parts a page read gives are named, so a caller reads them at once.
#[test]
fn every_guard_that_holds_a_change_back_is_named() {
    let mut unchecked = Unchecked::all();
    let mut changes = ChangeSet::default();
    let mut snapshot = without_page_reads();
    snapshot.insert(LinuxPackages::default());
    changes.want::<AccountLocks>(user_name("nico"), Toggle::On, &Snapshot::default());
    changes.want::<AurPackages>(package("yay-bin"), Queued, &Snapshot::default());

    unchecked.recheck(&mut changes, &snapshot);

    assert_eq!(
        unchecked.holding(&changes).collect::<Vec<_>>(),
        ["user-locks", "aur-packages"]
    );
}
