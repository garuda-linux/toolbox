mod run_now;

use toolbox_plan::{PlannedAction, RemoteAction, RootAction, TargetList, Upgrader, UserAction};

use super::{
    Operation, Operations, Task, Tool, plan_aur_upgrade, plan_health_check, plan_health_fix,
    plan_now, plan_remote, upgrader_for,
};
use crate::domain::Summary;
use crate::domains::InstalledPackages;
use crate::test_support::{package, packages, restored};
use crate::{ChangeSet, Snapshot, domains, plan};

const UPGRADE: Task = Task::UpgradeSystem {
    upgrader: Upgrader::GarudaUpdate,
};

fn installed(names: &[&str]) -> InstalledPackages {
    InstalledPackages(packages(names))
}

fn queue(tasks: &[Task]) -> ChangeSet {
    let mut changes = ChangeSet::default();
    for task in tasks {
        Operations::want(&mut changes, *task, &Snapshot::default());
    }

    changes
}

fn root_actions(changes: &ChangeSet) -> Vec<RootAction> {
    plan(changes)
        .steps()
        .iter()
        .map(|step| match &step.action {
            PlannedAction::Root(action) => action.clone(),
            PlannedAction::User(action) => panic!("unexpected user step {action:?}"),
        })
        .collect()
}

#[test]
fn queued_operations_run_in_their_fixed_order() {
    let changes = queue(&[
        Task::CleanPackageCache,
        UPGRADE,
        Task::RemoveOrphans,
        Task::RefreshKeyring,
        Task::RemoveDatabaseLock,
    ]);

    assert_eq!(
        root_actions(&changes),
        [
            RootAction::RefreshKeyring,
            RootAction::RemoveDatabaseLock,
            RootAction::UpgradeSystem {
                upgrader: Upgrader::GarudaUpdate
            },
            RootAction::RemoveOrphans,
            RootAction::CleanPackageCache,
        ]
    );
}

#[test]
fn an_operation_stays_queued_until_unqueued() {
    let mut changes = queue(&[Task::CleanPackageCache]);

    changes.reconcile(&Snapshot::default());
    assert_eq!(changes.len(), 1);

    changes.remove::<Operations>(&Operation::CleanPackageCache);
    assert!(changes.is_empty());
}

#[test]
fn unqueuing_what_is_not_queued_leaves_nothing() {
    let mut changes = ChangeSet::default();

    changes.remove::<Operations>(&Operation::RemoveOrphans);

    assert!(changes.is_empty());
}

#[test]
fn a_successful_apply_empties_the_operations_queue() {
    let mut changes = queue(&[UPGRADE, Task::RemoveOrphans]);

    crate::test_support::all_steps_succeeded(&mut changes);

    assert!(changes.is_empty());
}

/// The upgrade ran, the orphan removal after it failed: only the upgrade
/// leaves the queue, so the next apply does not upgrade again.
#[test]
fn only_one_offs_whose_steps_succeeded_leave_the_queue() {
    let mut changes = queue(&[UPGRADE, Task::RemoveOrphans]);
    let upgrade = plan(&queue(&[UPGRADE]));
    let done: Vec<PlannedAction> = upgrade
        .steps()
        .iter()
        .map(|step| step.action.clone())
        .collect();

    changes.drop_done_one_offs(&done);

    assert_eq!(changes.len(), 1);
    assert!(
        changes
            .wish::<Operations>(&Operation::RemoveOrphans)
            .is_some()
    );
}

#[test]
fn a_queued_upgrade_replaces_the_one_queued_before() {
    let pacman = Task::UpgradeSystem {
        upgrader: Upgrader::Pacman,
    };

    let mut changes = queue(&[UPGRADE, pacman]);

    assert_eq!(changes.len(), 1);
    assert_eq!(changes.views().len(), 1);
    assert_eq!(
        root_actions(&changes),
        [RootAction::UpgradeSystem {
            upgrader: Upgrader::Pacman
        }]
    );
    assert!(changes.remove_key("operations", "upgrade-system"));
    assert!(changes.is_empty());
}

#[test]
fn cleaning_up_runs_after_every_package_change() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(installed(&["vlc"]));
    let mut changes = queue(&[Task::CleanPackageCache, Task::RemoveOrphans, UPGRADE]);

    changes.want::<domains::Packages>(package("vlc"), crate::Toggle::Off, &snapshot);
    changes.want::<domains::Hblock>(crate::Single, crate::Toggle::On, &Snapshot::default());

    let one = |name: &str| TargetList::one(package(name));

    assert_eq!(
        root_actions(&changes),
        [
            RootAction::UpgradeSystem {
                upgrader: Upgrader::GarudaUpdate
            },
            RootAction::RemovePackages {
                packages: one("vlc")
            },
            RootAction::InstallPackages {
                packages: one("hblock")
            },
            RootAction::SetHblock { enabled: true },
            RootAction::RemoveOrphans,
            RootAction::CleanPackageCache,
        ]
    );
}

#[test]
fn operations_have_stable_kebab_case_keys() {
    let changes = queue(&[
        Task::RefreshKeyring,
        Task::RemoveDatabaseLock,
        UPGRADE,
        Task::RemoveOrphans,
        Task::CleanPackageCache,
    ]);

    let views: Vec<(&str, String, Summary)> = changes
        .views()
        .into_iter()
        .map(|view| (view.domain, view.key, view.summary))
        .collect();

    let operation = |key: &'static str| ("operations", key.to_owned(), Summary::plain(key));

    assert_eq!(
        views,
        [
            operation("refresh-keyring"),
            operation("remove-database-lock"),
            operation("upgrade-system"),
            operation("remove-orphans"),
            operation("clean-package-cache"),
        ]
    );
}

#[test]
fn queued_operations_survive_a_restart() {
    let changes = queue(&[UPGRADE, Task::CleanPackageCache]);
    let (restored, problems) = restored(&changes, &Snapshot::default());

    assert_eq!(problems, []);
    assert_eq!(restored.views(), changes.views());
    assert_eq!(
        serde_json::to_value(UPGRADE).unwrap(),
        serde_json::json!({"kind": "upgrade-system", "upgrader": "garuda-update"})
    );
    assert_eq!(
        serde_json::to_value(Operation::UpgradeSystem).unwrap(),
        serde_json::json!("upgrade-system")
    );
}

/// A task names its operation itself, so one filed under another is
/// ignored, and refused when it comes from a saved queue.
#[test]
fn a_task_is_filed_under_its_own_operation() {
    let mut changes = ChangeSet::default();

    changes.want::<Operations>(Operation::RemoveOrphans, UPGRADE, &Snapshot::default());
    assert!(changes.is_empty());

    let saved = serde_json::from_value(serde_json::json!({
        "version": crate::SAVED_VERSION,
        "changes": [{
            "domain": "operations",
            "item": "remove-orphans",
            "wish": { "kind": "upgrade-system", "upgrader": "pacman" },
        }],
    }))
    .unwrap();

    let problems = domains::all()
        .unwrap()
        .restore(&mut changes, &saved, &Snapshot::default());

    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(changes.is_empty());
}

#[test]
fn garuda_update_is_used_where_it_is_installed() {
    assert_eq!(
        upgrader_for(&installed(&["garuda-update", "pacman"])),
        Upgrader::GarudaUpdate
    );
    assert_eq!(upgrader_for(&installed(&["pacman"])), Upgrader::Pacman);
}

#[test]
fn tools_are_fixed_programs_from_known_packages() {
    let tools: Vec<(&str, &str, &[&str], String)> = Tool::ALL
        .into_iter()
        .map(|tool| {
            (
                tool.id(),
                tool.program(),
                tool.args(),
                tool.package().to_string(),
            )
        })
        .collect();

    assert_eq!(
        tools,
        [
            (
                "arch-mirrors",
                "/usr/bin/reflector-simple",
                [].as_slice(),
                "reflector-simple".to_owned()
            ),
            (
                "btrfs-assistant",
                "/usr/bin/btrfs-assistant-launcher",
                [].as_slice(),
                "btrfs-assistant".to_owned()
            ),
            (
                "edit-repositories",
                "/usr/bin/pace",
                [].as_slice(),
                "pace".to_owned()
            ),
            (
                "merge-pacdiff",
                "/usr/lib/garuda/launch-terminal",
                ["/usr/lib/garuda/pacdiff-merge"].as_slice(),
                "meld".to_owned()
            ),
        ]
    );
}

#[test]
fn tools_are_found_by_id() {
    for tool in Tool::ALL {
        assert_eq!(Tool::from_id(tool.id()), Some(tool));
    }

    assert_eq!(Tool::from_id("bash"), None);
}

/// Saved queues and self-tests name tools by these ids.
#[test]
fn tool_ids_stay_the_same() {
    let ids: Vec<_> = Tool::ALL.into_iter().map(Tool::id).collect();

    assert_eq!(
        ids,
        [
            "arch-mirrors",
            "btrfs-assistant",
            "edit-repositories",
            "merge-pacdiff",
        ]
    );
}

#[test]
fn one_operation_can_run_at_once_without_the_queue() {
    let plan = plan_now(Task::RemoveOrphans);

    assert_eq!(plan.steps().len(), 1);
    assert_eq!(
        plan.steps()[0].action,
        PlannedAction::Root(RootAction::RemoveOrphans)
    );
}

#[test]
fn repairs_and_health_checks_run_at_once() {
    let remote = plan_remote(RemoteAction::ResetSnapper);
    let fix = plan_health_fix();
    let check = plan_health_check();

    assert_eq!(
        remote.steps()[0].action,
        PlannedAction::Root(RootAction::GarudaUpdateRemote {
            action: RemoteAction::ResetSnapper
        })
    );
    assert_eq!(
        fix.steps()[0].action,
        PlannedAction::Root(RootAction::GarudaHealthFix)
    );
    assert_eq!(
        check.steps()[0].action,
        PlannedAction::User(UserAction::GarudaHealth)
    );
    assert!(
        [&remote, &fix, &check]
            .iter()
            .all(|plan| plan.steps().len() == 1)
    );
}

#[test]
fn the_aur_upgrade_runs_as_the_user() {
    assert_eq!(
        plan_aur_upgrade().steps()[0].action,
        PlannedAction::User(UserAction::UpgradeAur)
    );
}

#[test]
fn the_upgrader_package_is_garuda_update() {
    assert_eq!(super::GARUDA_UPDATE_PACKAGE.as_str(), "garuda-update");
}

#[test]
fn the_ranking_tool_has_a_valid_package_name() {
    assert_eq!(super::RATE_MIRRORS_PACKAGE.as_str(), "rate-mirrors");
}
