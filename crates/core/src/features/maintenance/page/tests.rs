use std::collections::BTreeSet;
use std::path::Path;

use toolbox_plan::{PackageName, PlannedAction, RemoteAction, RootAction, Upgrader};

use super::{
    OperationRow, PageTool, Repair, ToolRow, asks_first, install, installed, missing, operations,
    repairs, rescue_install, run_now_plan, task, tools, upgrader,
};
use crate::domains::InstalledPackages;
use crate::features::maintenance::{Operation, Operations, Tool};
use crate::test_support::package;
use crate::{ChangeSet, Snapshot};

fn root_action(id: &str) -> Option<RootAction> {
    let plan = run_now_plan(id, Upgrader::Pacman)?;

    match &plan.steps().first()?.action {
        PlannedAction::Root(action) => Some(action.clone()),
        PlannedAction::User(_) => None,
    }
}

#[test]
fn garuda_update_upgrades_when_it_is_installed() {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([package(
        "garuda-update",
    )])));

    assert_eq!(upgrader(&snapshot, false), Upgrader::GarudaUpdate);
    assert_eq!(upgrader(&Snapshot::default(), true), Upgrader::GarudaUpdate);
    assert_eq!(upgrader(&Snapshot::default(), false), Upgrader::Pacman);
}

#[test]
fn run_now_buttons_map_to_one_step_plans() {
    assert_eq!(
        root_action("clean-package-cache"),
        Some(RootAction::CleanPackageCache)
    );
    assert_eq!(
        root_action("remote-full-fix"),
        Some(RootAction::GarudaUpdateRemote {
            action: RemoteAction::FullFix
        })
    );
    assert_eq!(root_action("health-fix"), Some(RootAction::GarudaHealthFix));
    assert!(
        run_now_plan("health-check", Upgrader::Pacman).is_some(),
        "the check runs as the user"
    );
    assert!(
        run_now_plan("aur-upgrade", Upgrader::Pacman).is_some()
            && root_action("aur-upgrade").is_none(),
        "paru runs as the user"
    );
    assert_eq!(
        root_action("chroot"),
        Some(RootAction::ChrootInstallation),
        "the chroot needs root"
    );
    assert!(run_now_plan("rm-rf", Upgrader::Pacman).is_none());
}

#[test]
fn queued_operations_are_marked() {
    let mut changes = ChangeSet::default();
    let snapshot = Snapshot::default();
    Operations::want(
        &mut changes,
        task("remove-orphans", Upgrader::Pacman).unwrap(),
        &snapshot,
    );

    let shown = operations(&changes);

    assert_eq!(
        shown.get(1),
        Some(&OperationRow {
            operation: Operation::RemoveOrphans,
            queued: true
        })
    );
    assert_eq!(
        shown.first(),
        Some(&OperationRow {
            operation: Operation::UpgradeSystem,
            queued: false
        })
    );
}

#[test]
fn the_operations_come_in_the_order_of_the_page() {
    let listed: Vec<&str> = operations(&ChangeSet::default())
        .iter()
        .map(|row| row.operation.key())
        .collect();

    assert_eq!(
        listed,
        [
            "upgrade-system",
            "remove-orphans",
            "clean-package-cache",
            "refresh-keyring",
            "remove-database-lock",
        ]
    );
}

#[test]
fn tools_follow_the_installed_programs() {
    let installed = |path: &Path| path == Path::new("/usr/bin/pace");
    let tools = tools(installed);

    assert!(tools.contains(&ToolRow {
        tool: PageTool::Program(Tool::EditRepositories),
        installed: true
    }));
    assert!(tools.contains(&ToolRow {
        tool: PageTool::ChaoticMirrors,
        installed: false
    }));
}

#[test]
fn the_tools_come_in_the_order_of_the_page() {
    let listed: Vec<&str> = tools(|_| false).iter().map(|row| row.tool.key()).collect();

    assert_eq!(
        listed,
        [
            "arch-mirrors",
            "chaotic-mirrors",
            "btrfs-assistant",
            "edit-repositories",
            "merge-pacdiff",
        ]
    );
}

#[test]
fn repairs_follow_the_installed_programs() {
    let repairs = repairs(|path| path == Path::new("/usr/bin/garuda-update"));
    let available = |repair: &str| {
        repairs
            .iter()
            .find(|row| row.repair.key() == repair)
            .map(|row| row.available)
    };

    assert_eq!(available("remote-reset-audio"), Some(true));
    assert_eq!(available("health-check"), Some(false));
    assert_eq!(available("chroot"), None, "the Home page offers the chroot");
}

#[test]
fn the_repairs_come_in_the_order_of_the_page() {
    let listed: Vec<&str> = repairs(|_| true)
        .into_iter()
        .map(|row| row.repair.key())
        .collect();

    assert_eq!(
        listed,
        [
            "health-check",
            "health-fix",
            "remote-fix",
            "remote-keyring",
            "remote-full-fix",
            "remote-reset-audio",
            "remote-reset-snapper",
            "remote-reinstall",
        ]
    );
}

fn installed_by(plan: &toolbox_plan::Plan) -> Vec<&str> {
    match plan.steps().first().map(|step| &step.action) {
        Some(PlannedAction::Root(RootAction::InstallPackages { packages })) => {
            packages.iter().map(PackageName::as_str).collect()
        }
        Some(PlannedAction::Root(_) | PlannedAction::User(_)) | None => Vec::new(),
    }
}

#[test]
fn a_missing_garuda_update_is_installed_by_one_step() {
    let rows = repairs(|path| path != Path::new(super::GARUDA_UPDATE_PROGRAM));
    let plan = rescue_install(&rows).expect("a missing garuda-update has a plan");

    assert_eq!(plan.steps().len(), 1);
    assert_eq!(installed_by(&plan), ["garuda-update"]);
}

#[test]
fn every_missing_package_is_named_once_and_installed_by_one_step() {
    let rows = repairs(|_| false);
    let named: Vec<&str> = missing(&rows)
        .into_iter()
        .map(PackageName::as_str)
        .collect();

    let plan = rescue_install(&rows).expect("missing packages have a plan");

    assert_eq!(named, ["garuda-health", "garuda-update"]);
    assert_eq!(plan.steps().len(), 1);
    assert_eq!(installed_by(&plan), ["garuda-health", "garuda-update"]);
}

#[test]
fn a_missing_garuda_health_alone_is_installed_too() {
    let rows = repairs(|path| path == Path::new(super::GARUDA_UPDATE_PROGRAM));
    let plan = rescue_install(&rows).expect("a missing garuda-health has a plan");

    assert_eq!(installed_by(&plan), ["garuda-health"]);
}

#[test]
fn installed_repair_programs_need_no_install() {
    assert!(rescue_install(&repairs(|_| true)).is_none());
}

#[test]
fn a_missing_tool_is_installed_by_one_step() {
    let plan = install(&super::super::RATE_MIRRORS_PACKAGE);
    let step = plan.steps().first().map(|step| &step.action);

    assert!(matches!(
        step,
        Some(PlannedAction::Root(RootAction::InstallPackages { packages }))
            if packages.iter().map(PackageName::as_str).eq(["rate-mirrors"])
    ));
}

#[test]
fn a_tool_counts_as_installed_only_when_its_row_found_it() {
    let rows = tools(|path| path == Path::new(super::RATE_MIRRORS_PROGRAM));

    assert!(installed(&rows, PageTool::ChaoticMirrors));
    assert!(!installed(&rows, PageTool::Program(Tool::BtrfsAssistant)));
}

#[test]
fn repairs_that_reinstall_or_wipe_ask_first() {
    let asking: Vec<&str> = repairs(|_| true)
        .into_iter()
        .map(|row| row.repair)
        .filter(|repair| repair.asks_first() && asks_first(repair.key()))
        .map(Repair::key)
        .collect();

    assert_eq!(
        asking,
        [
            "remote-fix",
            "remote-full-fix",
            "remote-reset-audio",
            "remote-reset-snapper",
            "remote-reinstall",
        ]
    );
}

#[test]
fn removing_orphans_and_cleaning_the_cache_ask_first() {
    assert!(asks_first("remove-orphans"));
    assert!(asks_first("clean-package-cache"));
    assert!(!asks_first(Operation::UpgradeSystem.key()));
    assert!(!asks_first("refresh-keyring"));
}

#[test]
fn each_repair_names_the_program_it_needs() {
    let rows = repairs(|_| false);

    let needs = |id: &str| {
        rows.iter()
            .find(|row| row.repair.key() == id)
            .unwrap()
            .needs
            .to_string()
    };

    assert_eq!(needs("health-check"), "garuda-health");
    assert_eq!(needs("remote-fix"), "garuda-update");
}

#[test]
fn every_repair_has_its_key_and_the_text_of_the_repairs_tab() {
    for repair in Repair::ALL {
        assert_eq!(Repair::from_key(repair.key()), Some(repair));
    }

    assert_eq!(
        Repair::Remote(RemoteAction::FullFix).text_id(),
        "maintenance-repair-remote-full-fix"
    );
    assert_eq!(
        Repair::HealthCheck.text_id(),
        "maintenance-repair-health-check"
    );
    assert_eq!(Repair::from_key("chroot"), None);
}

#[test]
fn a_typed_repair_plans_its_one_step() {
    let action = |repair: Repair| match &repair.plan().steps().first().unwrap().action {
        PlannedAction::Root(action) => Some(action.clone()),
        PlannedAction::User(_) => None,
    };

    assert_eq!(
        action(Repair::Remote(RemoteAction::Keyring)),
        Some(RootAction::GarudaUpdateRemote {
            action: RemoteAction::Keyring
        })
    );
    assert_eq!(action(Repair::HealthFix), Some(RootAction::GarudaHealthFix));
    assert_eq!(
        action(Repair::HealthCheck),
        None,
        "the check runs as the user"
    );
}

#[test]
fn an_operation_and_a_tool_have_the_texts_of_the_common_tab() {
    assert_eq!(
        Operation::RemoveOrphans.text_id(),
        "maintenance-op-remove-orphans"
    );
    assert!(Operation::CleanPackageCache.asks_first());
    assert!(!Operation::UpgradeSystem.asks_first());
    assert_eq!(
        PageTool::ChaoticMirrors.text_id(),
        "maintenance-tool-chaotic-mirrors"
    );
    assert_eq!(
        PageTool::Program(Tool::BtrfsAssistant).text_id(),
        "maintenance-tool-btrfs-assistant"
    );
}

/// pacdiff-merge needs meld, and launch-terminal is on every Garuda
/// system, so the launcher alone must not count as the tool.
#[test]
fn the_merge_tool_counts_as_installed_only_with_meld() {
    let without = tools(|path| path != Path::new("/usr/bin/meld"));
    let with = tools(|_| true);

    assert!(!installed(&without, PageTool::Program(Tool::MergePacdiff)));
    assert!(installed(&with, PageTool::Program(Tool::MergePacdiff)));
}
