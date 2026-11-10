use std::collections::BTreeSet;
use std::path::Path;

use toolbox_plan::{PackageName, PlannedAction, RemoteAction, RootAction, Upgrader};

use super::{
    LaunchCause, OperationRow, ToolRow, UPGRADE_SYSTEM, asks_first, install, installed, operations,
    repairs, rescue_install, run_now_plan, task, tools, upgrader,
};
use crate::domains::InstalledPackages;
use crate::features::maintenance::Operations;
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
            operation: "remove-orphans",
            queued: true
        })
    );
    assert_eq!(
        shown.first(),
        Some(&OperationRow {
            operation: "upgrade-system",
            queued: false
        })
    );
}

#[test]
fn the_operations_come_in_the_order_of_the_page() {
    let listed: Vec<&str> = operations(&ChangeSet::default())
        .iter()
        .map(|row| row.operation)
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
        tool: "edit-repositories",
        installed: true
    }));
    assert!(tools.contains(&ToolRow {
        tool: "chaotic-mirrors",
        installed: false
    }));
}

#[test]
fn the_tools_come_in_the_order_of_the_page() {
    let listed: Vec<&str> = tools(|_| false).iter().map(|row| row.tool).collect();

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
            .find(|row| row.repair == repair)
            .map(|row| row.available)
    };

    assert_eq!(available("remote-reset-audio"), Some(true));
    assert_eq!(available("health-check"), Some(false));
    assert_eq!(available("chroot"), None, "the Home page offers the chroot");
}

#[test]
fn the_repairs_come_in_the_order_of_the_page() {
    let listed: Vec<String> = repairs(|_| true)
        .into_iter()
        .map(|row| row.repair)
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

#[test]
fn a_missing_garuda_update_is_installed_by_one_step() {
    let plan = rescue_install(false).expect("a missing garuda-update has a plan");
    let step = plan.steps().first().map(|step| &step.action);

    assert_eq!(plan.steps().len(), 1);
    assert!(matches!(
        step,
        Some(PlannedAction::Root(RootAction::InstallPackages { packages }))
            if packages.iter().map(PackageName::as_str).eq(["garuda-update"])
    ));
}

#[test]
fn an_installed_garuda_update_needs_no_install() {
    assert!(rescue_install(true).is_none());
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

    assert!(installed(&rows, super::CHAOTIC_MIRRORS));
    assert!(!installed(&rows, "btrfs-assistant"));
    assert!(!installed(&rows, "no-such-tool"));
}

#[test]
fn a_start_failure_says_whether_the_program_is_missing_or_forbidden() {
    use std::io::ErrorKind;

    assert_eq!(LaunchCause::of(ErrorKind::NotFound), LaunchCause::Missing);
    assert_eq!(
        LaunchCause::of(ErrorKind::PermissionDenied),
        LaunchCause::Denied
    );
    assert_eq!(LaunchCause::of(ErrorKind::OutOfMemory), LaunchCause::Other);
}

#[test]
fn repairs_that_reinstall_or_wipe_ask_first() {
    let asking: Vec<String> = repairs(|_| true)
        .into_iter()
        .map(|row| row.repair)
        .filter(|repair| asks_first(repair))
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
    assert!(!asks_first(UPGRADE_SYSTEM));
    assert!(!asks_first("refresh-keyring"));
}

#[test]
fn each_repair_names_the_program_it_needs() {
    let rows = repairs(|_| false);

    let needs = |id: &str| rows.iter().find(|row| row.repair == id).unwrap().needs;
    assert_eq!(needs("health-check"), "garuda-health");
    assert_eq!(needs("remote-fix"), "garuda-update");
}
