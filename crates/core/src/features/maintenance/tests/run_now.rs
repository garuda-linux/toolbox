use std::collections::BTreeSet;

use toolbox_plan::{Plan, RootAction, Upgrader, UserAction};

use crate::domains::InstalledPackages;
use crate::features::maintenance::{GARUDA_UPDATE_PACKAGE, RunNow, queue_upgrade, upgrade_queued};
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
    let garuda = RunNow::UpgradeSystem.plan(&with_garuda_update(), false);

    assert_eq!(
        garuda,
        Plan::root(RootAction::UpgradeSystem {
            upgrader: Upgrader::GarudaUpdate,
        })
    );

    let pacman = RunNow::UpgradeSystem.plan(&Snapshot::default(), false);
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
        RunNow::AurUpgrade.plan(&snapshot, false),
        Plan::user(UserAction::UpgradeAur)
    );
    assert_eq!(
        RunNow::HealthCheck.plan(&snapshot, false),
        Plan::user(UserAction::GarudaHealth)
    );
    assert_eq!(
        RunNow::HealthFix.plan(&snapshot, false),
        Plan::root(RootAction::GarudaHealthFix)
    );
    assert!(!RunNow::Chroot.plan(&snapshot, false).is_empty());
}

#[test]
fn the_update_queues_and_goes_back() {
    let snapshot = with_garuda_update();
    let mut changes = ChangeSet::default();

    queue_upgrade(true, &mut changes, &snapshot, false);
    assert!(upgrade_queued(&changes));

    queue_upgrade(false, &mut changes, &snapshot, false);
    assert!(!upgrade_queued(&changes));
    assert!(changes.is_empty());
}

#[test]
fn without_the_package_list_the_update_takes_garuda_update_when_its_program_is_there() {
    let plan = RunNow::UpgradeSystem.plan(&Snapshot::default(), true);

    assert_eq!(
        plan,
        Plan::root(RootAction::UpgradeSystem {
            upgrader: Upgrader::GarudaUpdate,
        })
    );
}
