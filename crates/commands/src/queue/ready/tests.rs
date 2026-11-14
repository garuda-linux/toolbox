use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use toolbox_cli::Exit;
use toolbox_core::domains::{
    AccountLockFlags, AccountLocks, Accounts, AurPackages, CurrentLoader, CurrentUser,
    LoaderSettingsDomain,
};
use toolbox_core::features::packages::aur::AurNames;
use toolbox_core::features::{Guard, Refusal};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Queued, SavedChanges, Snapshot, Toggle};
use toolbox_plan::sdboot::{LoaderSetting, LoaderSettings};
use toolbox_plan::{PackageName, UserName};
use toolbox_services::unprobed::UnprobedReads;

use super::{after, checked, refused};
use crate::out::{Out, Settings};

/// Answers what it was given and counts how often it was asked.
#[derive(Default)]
struct Reads {
    locks: Option<AccountLockFlags>,
    names: Option<AurNames>,
    loader: Option<CurrentLoader>,
    asked_locks: Cell<usize>,
    asked_names: Cell<usize>,
    asked_loader: Cell<usize>,
}

impl UnprobedReads for Reads {
    fn locks(&self) -> Option<AccountLockFlags> {
        self.asked_locks.set(self.asked_locks.get() + 1);
        self.locks.clone()
    }

    fn aur_names(&self) -> Option<AurNames> {
        self.asked_names.set(self.asked_names.get() + 1);
        self.names.clone()
    }

    fn loader(&self) -> Option<CurrentLoader> {
        self.asked_loader.set(self.asked_loader.get() + 1);
        self.loader.clone()
    }
}

fn loader_change() -> ChangeSet {
    let mut changes = ChangeSet::default();
    LoaderSettingsDomain::want(
        &mut changes,
        LoaderSetting::Timeout(None),
        &Snapshot::default(),
    );
    changes
}

fn user(name: &str) -> UserName {
    UserName::new(name).unwrap()
}

/// What the probe gives the account guards, without the lock flags.
fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts::default());
    snapshot.insert(CurrentUser(user("nico")));
    snapshot
}

/// A lock of an account the system does not have, so a guard that runs
/// drops it.
fn lock() -> SavedChanges {
    let mut changes = ChangeSet::default();
    changes.want::<AccountLocks>(user("ghost"), Toggle::On, &Snapshot::default());
    changes.to_saved()
}

fn build() -> SavedChanges {
    let mut changes = ChangeSet::default();
    let package = PackageName::new("yay-bin").unwrap();
    changes.want::<AurPackages>(package, Queued, &Snapshot::default());
    changes.to_saved()
}

#[test]
fn a_restored_lock_reads_the_flags_and_meets_its_guard() {
    let reads = Reads {
        locks: Some(AccountLockFlags(BTreeMap::new())),
        ..Reads::default()
    };

    let changes = checked(&lock(), &system(), &reads).unwrap();

    assert!(changes.is_empty(), "the guard drops the unknown account");
    assert_eq!(reads.asked_locks.get(), 1);
    assert_eq!(reads.asked_names.get(), 0);
}

/// Fails closed: without the flags nothing checked the lock, so nothing runs.
#[test]
fn a_restored_lock_without_the_flags_is_refused() {
    let reads = Reads::default();

    assert_eq!(
        checked(&lock(), &system(), &reads).err(),
        Some(Refusal::GuardsWaiting {
            guard: Guard::UserLocks
        })
    );
}

#[test]
fn a_restored_build_reads_the_names_of_the_aur() {
    let reads = Reads {
        names: Some(AurNames(BTreeSet::new())),
        ..Reads::default()
    };

    let changes = checked(&build(), &system(), &reads).unwrap();

    assert!(changes.is_empty(), "the AUR has no such package");
    assert_eq!(reads.asked_names.get(), 1);
    assert_eq!(reads.asked_locks.get(), 0);
}

/// No guard checks the loader, so `checked` leaves the helper alone.
#[test]
fn a_restored_loader_change_reads_nothing_before_the_run() {
    let reads = Reads::default();

    let changes = checked(&loader_change().to_saved(), &system(), &reads).unwrap();

    assert_eq!(changes.len(), 1);
    assert_eq!(reads.asked_loader.get(), 0);
}

#[test]
fn after_a_run_of_a_loader_change_the_loader_joins_the_snapshot() {
    let reads = Reads {
        loader: Some(CurrentLoader(LoaderSettings::default())),
        ..Reads::default()
    };

    let snapshot = after(system(), &loader_change(), &reads);

    assert!(snapshot.part::<CurrentLoader>().is_some());
    assert_eq!(reads.asked_loader.get(), 1);
    assert_eq!(reads.asked_locks.get() + reads.asked_names.get(), 0);
}

#[test]
fn after_a_run_without_a_loader_change_nothing_is_read() {
    let reads = Reads::default();

    after(system(), &ChangeSet::default(), &reads);

    assert_eq!(reads.asked_loader.get(), 0);
}

#[test]
fn a_refusal_names_what_is_missing_and_exits_unavailable() {
    let mut out = Vec::new();

    let exit = refused(
        Refusal::GuardsWaiting {
            guard: Guard::AurPackages,
        },
        &mut Out::new(&mut out, Settings::default()),
    );

    let said = String::from_utf8(out).unwrap();
    assert_eq!(exit, Exit::Unavailable);
    assert!(said.contains("package list of the AUR"), "{said}");
    assert!(said.contains("queue remove"), "{said}");
}
