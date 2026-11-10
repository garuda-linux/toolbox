use std::collections::BTreeSet;

use toolbox_plan::{Plan, RootAction, Toggle, Upgrader, UserAction};

use crate::domains::InstalledPackages;
use crate::features::maintenance::{
    GARUDA_UPDATE_PACKAGE, Program, RunNow, queue_upgrade, upgrade_queued,
};
use crate::{ChangeSet, Snapshot};

fn with_garuda_update() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([
        GARUDA_UPDATE_PACKAGE.clone()
    ])));
    snapshot
}

#[test]
fn the_update_runs_with_garuda_update_where_it_is_installed() {
    let garuda = RunNow::UpgradeSystem.plan(&with_garuda_update(), Program::Missing);

    assert_eq!(
        garuda,
        Plan::root(RootAction::UpgradeSystem {
            upgrader: Upgrader::GarudaUpdate,
        })
    );

    let pacman = RunNow::UpgradeSystem.plan(&Snapshot::default(), Program::Missing);
    assert_eq!(
        pacman,
        Plan::root(RootAction::UpgradeSystem {
            upgrader: Upgrader::Pacman,
        })
    );
}

#[test]
fn every_button_has_its_plan() {
    let snapshot = Snapshot::default();

    assert_eq!(
        RunNow::AurUpgrade.plan(&snapshot, Program::Missing),
        Plan::user(UserAction::UpgradeAur)
    );
    assert_eq!(
        RunNow::HealthCheck.plan(&snapshot, Program::Missing),
        Plan::user(UserAction::GarudaHealth)
    );
    assert_eq!(
        RunNow::HealthFix.plan(&snapshot, Program::Missing),
        Plan::root(RootAction::GarudaHealthFix)
    );
    assert!(!RunNow::Chroot.plan(&snapshot, Program::Missing).is_empty());
}

#[test]
fn the_update_queues_and_goes_back() {
    let snapshot = with_garuda_update();
    let mut changes = ChangeSet::default();

    queue_upgrade(Toggle::On, &mut changes, &snapshot, Program::Missing);
    assert!(upgrade_queued(&changes));

    queue_upgrade(Toggle::Off, &mut changes, &snapshot, Program::Missing);
    assert!(!upgrade_queued(&changes));
    assert!(changes.is_empty());
}

#[test]
fn without_the_package_list_the_update_takes_garuda_update_when_its_program_is_there() {
    let plan = RunNow::UpgradeSystem.plan(&Snapshot::default(), Program::Found);

    assert_eq!(
        plan,
        Plan::root(RootAction::UpgradeSystem {
            upgrader: Upgrader::GarudaUpdate,
        })
    );
}
