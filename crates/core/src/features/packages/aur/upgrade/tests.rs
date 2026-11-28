//! The orders come from the Lima captures of `paru -Po --aur` for cbonsai
//! and pipes.sh, read with the installed names the test gives.

use toolbox_plan::{
    BuildRun, InstallReason, PackageName, PlannedAction, RootAction, TargetList, UserAction,
};
use toolbox_state::aur::order::parse_order;
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_state::testing::aur::order::{
    CBONSAI_OUTDATED_MAKE_STDOUT, CBONSAI_OUTDATED_STDOUT, PIPES_SH_CURRENT_STDOUT,
};

use super::{Start, UpgradePath, plan_upgrade, start, upgrade_path};
use crate::Snapshot;
use crate::domains::{BuildOrder, InstalledPackages};
use crate::test_support::{package, packages};

fn run() -> BuildRun {
    BuildRun::new("2c3d").unwrap()
}

/// The order of `text` as read with `installed` names installed.
fn order(text: &str, installed: &[&str]) -> BuildOrder {
    BuildOrder {
        order: parse_order(text).unwrap(),
        run: run(),
        installed: packages(installed),
    }
}

fn accepted() -> AurRoute {
    AurRoute::Accepted
}

fn refused() -> AurRoute {
    AurRoute::Refused {
        kind: RouteKind::Linked,
        detail: "a folder on the way is missing".to_owned(),
    }
}

/// The system with the route and the installed packages read.
fn system(route: AurRoute, installed: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(route);
    snapshot.insert(InstalledPackages(packages(installed)));
    snapshot
}

fn targets(names: &[&str]) -> TargetList<PackageName> {
    TargetList::new(names.iter().map(|name| package(name))).unwrap()
}

fn root(action: RootAction) -> PlannedAction {
    PlannedAction::Root(action)
}

fn user(action: UserAction) -> PlannedAction {
    PlannedAction::User(action)
}

/// The names an action installs, removes, builds or reviews.
fn named(action: &PlannedAction) -> Vec<PackageName> {
    match action {
        PlannedAction::Root(
            RootAction::InstallPackages { packages }
            | RootAction::InstallDependencies { packages }
            | RootAction::RemoveMakeDependencies { packages }
            | RootAction::InstallBuilt {
                run: _,
                packages,
                reason: _,
            },
        ) => packages.iter().cloned().collect(),
        PlannedAction::User(UserAction::BuildAur { base, run: _ }) => vec![base.clone()],
        PlannedAction::User(UserAction::ReviewAur { bases }) => bases.iter().cloned().collect(),
        PlannedAction::Root(_) | PlannedAction::User(_) => Vec::new(),
    }
}

#[test]
fn an_outdated_package_is_built_through_the_split_path() {
    let snapshot = system(accepted(), &["cbonsai"]);
    let path = upgrade_path(&snapshot, Some(&[package("cbonsai")]));
    assert_eq!(path, UpgradePath::Split(vec![package("cbonsai")]));

    let upgrade = plan_upgrade(path, Some(&[order(CBONSAI_OUTDATED_STDOUT, &["cbonsai"])]));

    assert_eq!(upgrade.path, UpgradePath::Split(vec![package("cbonsai")]));
    assert_eq!(
        upgrade.actions,
        [
            user(UserAction::ReviewAur {
                bases: targets(&["cbonsai"]),
            }),
            user(UserAction::BuildAur {
                base: package("cbonsai"),
                run: run(),
            }),
            root(RootAction::InstallBuilt {
                run: run(),
                packages: targets(&["cbonsai"]),
                reason: InstallReason::Keep,
            }),
        ]
    );
}

#[test]
fn a_missing_make_dependency_is_installed_first_and_removed_after_the_build() {
    let snapshot = system(accepted(), &["cbonsai"]);
    let path = upgrade_path(&snapshot, Some(&[package("cbonsai")]));

    let upgrade = plan_upgrade(
        path,
        Some(&[order(CBONSAI_OUTDATED_MAKE_STDOUT, &["cbonsai"])]),
    );

    assert_eq!(
        upgrade.actions,
        [
            user(UserAction::ReviewAur {
                bases: targets(&["cbonsai"]),
            }),
            root(RootAction::InstallDependencies {
                packages: targets(&["scdoc"]),
            }),
            user(UserAction::BuildAur {
                base: package("cbonsai"),
                run: run(),
            }),
            root(RootAction::InstallBuilt {
                run: run(),
                packages: targets(&["cbonsai"]),
                reason: InstallReason::Keep,
            }),
            root(RootAction::RemoveMakeDependencies {
                packages: targets(&["scdoc"]),
            }),
        ]
    );
}

#[test]
fn an_upgrade_keeps_the_install_reason_of_every_upgraded_package() {
    let snapshot = system(accepted(), &["cbonsai"]);
    let upgrade = plan_upgrade(
        upgrade_path(&snapshot, Some(&[package("cbonsai")])),
        Some(&[order(CBONSAI_OUTDATED_MAKE_STDOUT, &["cbonsai"])]),
    );

    let reasons: Vec<InstallReason> = upgrade
        .actions
        .iter()
        .filter_map(|action| match action {
            PlannedAction::Root(RootAction::InstallBuilt {
                run: _,
                packages,
                reason,
            }) if packages.iter().any(|name| name.as_str() == "cbonsai") => Some(*reason),
            PlannedAction::Root(_) | PlannedAction::User(_) => None,
        })
        .collect();

    assert_eq!(reasons, [InstallReason::Keep]);
}

#[test]
fn a_current_package_is_not_rebuilt() {
    let snapshot = system(accepted(), &["cbonsai", "pipes.sh"]);

    let nothing = upgrade_path(&snapshot, Some(&[]));
    assert_eq!(nothing, UpgradePath::UpToDate);
    assert_eq!(plan_upgrade(nothing, None).actions, Vec::new());

    let upgrade = plan_upgrade(
        upgrade_path(&snapshot, Some(&[package("cbonsai")])),
        Some(&[
            order(CBONSAI_OUTDATED_STDOUT, &["cbonsai"]),
            order(PIPES_SH_CURRENT_STDOUT, &["pipes.sh"]),
        ]),
    );

    assert!(
        upgrade
            .actions
            .iter()
            .all(|action| !named(action).contains(&package("pipes.sh"))),
        "{:?}",
        upgrade.actions
    );
    assert!(
        upgrade
            .actions
            .iter()
            .any(|action| named(action).contains(&package("cbonsai")))
    );
}

#[test]
fn nothing_outdated_plans_no_aur_step() {
    let snapshot = system(accepted(), &["cbonsai"]);

    assert_eq!(upgrade_path(&snapshot, Some(&[])), UpgradePath::UpToDate);
}

#[test]
fn a_refused_route_takes_paru_even_for_an_outdated_package() {
    let snapshot = system(refused(), &["cbonsai"]);

    let path = upgrade_path(&snapshot, Some(&[package("cbonsai")]));
    assert_eq!(path, UpgradePath::Paru);

    let upgrade = plan_upgrade(path, Some(&[order(CBONSAI_OUTDATED_STDOUT, &["cbonsai"])]));

    assert_eq!(upgrade.actions, [user(UserAction::UpgradeAur)]);
}

#[test]
fn a_refused_route_keeps_paru_when_nothing_is_outdated() {
    let snapshot = system(refused(), &["cbonsai"]);

    assert_eq!(upgrade_path(&snapshot, Some(&[])), UpgradePath::Paru);
}

#[test]
fn an_unread_route_or_installed_list_or_outdated_list_takes_paru() {
    let mut no_route = Snapshot::default();
    no_route.insert(InstalledPackages(packages(&["cbonsai"])));
    assert_eq!(
        upgrade_path(&no_route, Some(&[package("cbonsai")])),
        UpgradePath::Paru
    );

    let mut no_installed = Snapshot::default();
    no_installed.insert(accepted());
    assert_eq!(
        upgrade_path(&no_installed, Some(&[package("cbonsai")])),
        UpgradePath::Paru
    );

    let with_route = system(accepted(), &["cbonsai"]);
    assert_eq!(upgrade_path(&with_route, None), UpgradePath::Paru);
}

#[test]
fn an_order_that_is_not_read_takes_paru() {
    let upgrade = plan_upgrade(UpgradePath::Split(vec![package("cbonsai")]), None);

    assert_eq!(upgrade.path, UpgradePath::Paru);
    assert_eq!(upgrade.actions, [user(UserAction::UpgradeAur)]);
}

#[test]
fn a_package_whose_order_is_missing_takes_paru_for_the_whole_upgrade() {
    let snapshot = system(accepted(), &["cbonsai", "pipes.sh"]);

    let upgrade = plan_upgrade(
        upgrade_path(&snapshot, Some(&[package("cbonsai"), package("pipes.sh")])),
        Some(&[order(CBONSAI_OUTDATED_STDOUT, &["cbonsai"])]),
    );

    assert_eq!(upgrade.path, UpgradePath::Paru);
    assert_eq!(upgrade.actions, [user(UserAction::UpgradeAur)]);
}

#[test]
fn a_package_that_is_not_installed_takes_paru() {
    let snapshot = system(accepted(), &[]);

    let upgrade = plan_upgrade(
        upgrade_path(&snapshot, Some(&[package("cbonsai")])),
        Some(&[order(CBONSAI_OUTDATED_STDOUT, &[])]),
    );

    assert_eq!(upgrade.path, UpgradePath::Paru);
}

/// The person is there when the upgrade starts, so the review asks before the
/// long system upgrade, and a declined review changes nothing.
#[test]
fn the_review_comes_before_the_system_upgrade() {
    let snapshot = system(accepted(), &["cbonsai"]);
    let orders = [order(CBONSAI_OUTDATED_STDOUT, &["cbonsai"])];
    let upgrade = plan_upgrade(
        upgrade_path(&snapshot, Some(&[package("cbonsai")])),
        Some(&orders),
    );

    let system_step = root(RootAction::RefreshKeyring);
    let whole = upgrade.after(vec![system_step.clone()]);

    assert!(matches!(
        whole.first(),
        Some(PlannedAction::User(UserAction::ReviewAur { bases: _ }))
    ));
    assert_eq!(whole.get(1), Some(&system_step));
}

#[test]
fn parus_upgrade_comes_after_the_system_upgrade() {
    let snapshot = system(refused(), &["cbonsai"]);
    let upgrade = plan_upgrade(upgrade_path(&snapshot, Some(&[package("cbonsai")])), None);
    let system_step = root(RootAction::RefreshKeyring);

    assert_eq!(
        upgrade.after(vec![system_step.clone()]),
        [system_step, user(UserAction::UpgradeAur)]
    );
}

#[test]
fn an_outdated_package_on_the_split_path_waits_for_its_order() {
    let snapshot = system(accepted(), &["cbonsai"]);

    let Start::Read(waiting) = start(&snapshot, Some(&[package("cbonsai")])) else {
        panic!("the split path reads the order first");
    };

    assert_eq!(waiting.names(), [package("cbonsai")]);
    let upgrade = waiting.plan(Some(&[order(CBONSAI_OUTDATED_STDOUT, &["cbonsai"])]));
    assert_eq!(upgrade.path, UpgradePath::Split(vec![package("cbonsai")]));
    assert_eq!(
        upgrade.after(Vec::new()).first(),
        Some(&user(UserAction::ReviewAur {
            bases: targets(&["cbonsai"]),
        }))
    );
}

#[test]
fn a_waiting_upgrade_whose_order_is_not_read_takes_paru() {
    let snapshot = system(accepted(), &["cbonsai"]);

    let Start::Read(waiting) = start(&snapshot, Some(&[package("cbonsai")])) else {
        panic!("the split path reads the order first");
    };

    assert_eq!(waiting.plan(None).actions, [user(UserAction::UpgradeAur)]);
}

#[test]
fn a_refused_route_starts_paru_without_a_read() {
    let snapshot = system(refused(), &["cbonsai"]);

    let Start::Now(upgrade) = start(&snapshot, Some(&[package("cbonsai")])) else {
        panic!("a refused route reads nothing");
    };

    assert_eq!(upgrade.path, UpgradePath::Paru);
}

#[test]
fn nothing_outdated_starts_with_no_steps() {
    let snapshot = system(accepted(), &["cbonsai"]);

    let Start::Now(upgrade) = start(&snapshot, Some(&[])) else {
        panic!("nothing outdated reads nothing");
    };

    assert_eq!(upgrade.path, UpgradePath::UpToDate);
    assert_eq!(upgrade.actions, Vec::new());
}
