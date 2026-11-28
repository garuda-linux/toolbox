use std::collections::BTreeSet;

use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::packages::aur::upgrade::{AurUpgrade, UpgradePath};
use toolbox_plan::{InstallReason, PackageName, PlannedAction, RootAction, TargetList, UserAction};
use toolbox_services::aur_orders::RouteProblem;
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_state::testing::aur::order::{
    CBONSAI_OUTDATED_MAKE_STDOUT, CBONSAI_OUTDATED_STDOUT, PIPES_SH_CURRENT_STDOUT,
};
use toolbox_system::PendingUpdate;
use toolbox_system::testing::aur_orders::FakeAurOrders;
use toolbox_system::testing::aur_updates::FakeAurUpdates;

use super::aur_steps;

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

/// The pending update of `text`, from the version of the capture to the next.
fn update(text: &str) -> PendingUpdate {
    PendingUpdate {
        name: name(text),
        from: toolbox_text::plain("1.4.0-1"),
        to: toolbox_text::plain("1.4.2-1"),
    }
}

fn outdated(names: &[&str]) -> FakeAurUpdates {
    FakeAurUpdates::new(Ok(names.iter().map(|text| update(text)).collect()))
}

/// paru's order of cbonsai, its own build, as the captured `paru -Po` answered.
fn cbonsai_orders(text: &'static str) -> FakeAurOrders {
    FakeAurOrders::new(vec![("cbonsai", Ok(text))])
}

/// The system with cbonsai installed and the route answered.
fn system_with(route: AurRoute) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([name("cbonsai")])));
    snapshot.insert(route);
    snapshot
}

fn accepted() -> AurRoute {
    AurRoute::Accepted
}

/// The helper refuses this user's build folder.
fn refused() -> AurRoute {
    AurRoute::Refused {
        kind: RouteKind::Linked,
        detail: "a folder on the way is missing".to_owned(),
    }
}

/// The steps of `route` with `outdated` pending and paru answering `orders`.
fn steps(route: AurRoute, outdated: &FakeAurUpdates, orders: &FakeAurOrders) -> AurUpgrade {
    aur_steps(&system_with(route), outdated, orders, Ok(()), 1)
}

fn paru_upgrade() -> Vec<PlannedAction> {
    vec![PlannedAction::User(UserAction::UpgradeAur)]
}

#[test]
fn an_outdated_package_is_upgraded_through_the_split_path_with_its_review_first() {
    let orders = cbonsai_orders(CBONSAI_OUTDATED_STDOUT);

    let upgrade = steps(accepted(), &outdated(&["cbonsai"]), &orders);

    assert_eq!(upgrade.path, UpgradePath::Split(vec![name("cbonsai")]));
    assert!(
        matches!(
            upgrade.actions.first(),
            Some(PlannedAction::User(UserAction::ReviewAur { bases }))
                if bases.iter().map(PackageName::as_str).eq(["cbonsai"])
        ),
        "{:?}",
        upgrade.actions
    );
    assert!(upgrade.actions.iter().any(|action| matches!(
        action,
        PlannedAction::Root(RootAction::InstallBuilt {
            run: _,
            packages,
            reason: InstallReason::Keep,
        }) if packages.iter().map(PackageName::as_str).eq(["cbonsai"])
    )));
    assert_eq!(orders.asked(), 1);
}

#[test]
fn a_missing_make_dependency_is_installed_before_and_removed_after_the_build() {
    let orders = cbonsai_orders(CBONSAI_OUTDATED_MAKE_STDOUT);

    let upgrade = steps(accepted(), &outdated(&["cbonsai"]), &orders);

    assert!(upgrade.actions.iter().any(|action| matches!(
        action,
        PlannedAction::Root(RootAction::InstallDependencies { packages })
            if packages.iter().map(PackageName::as_str).eq(["scdoc"])
    )));
    assert_eq!(
        upgrade.actions.last(),
        Some(&PlannedAction::Root(RootAction::RemoveMakeDependencies {
            packages: TargetList::new(vec![name("scdoc")]).unwrap(),
        }))
    );
}

#[test]
fn a_current_package_is_neither_asked_for_nor_rebuilt() {
    let nothing = cbonsai_orders(CBONSAI_OUTDATED_STDOUT);
    let current = steps(accepted(), &outdated(&[]), &nothing);

    assert_eq!(current.path, UpgradePath::UpToDate);
    assert_eq!(current.actions, Vec::new());
    assert_eq!(nothing.asked(), 0);

    let orders = FakeAurOrders::new(vec![
        ("cbonsai", Ok(CBONSAI_OUTDATED_STDOUT)),
        ("pipes.sh", Ok(PIPES_SH_CURRENT_STDOUT)),
    ]);

    let upgrade = steps(accepted(), &outdated(&["cbonsai"]), &orders);

    assert_eq!(orders.asked(), 1);
    assert!(upgrade.actions.iter().all(|action| !matches!(
        action,
        PlannedAction::User(UserAction::BuildAur { base, run: _ }) if base.as_str() == "pipes.sh"
    )));
}

#[test]
fn a_refused_route_takes_paru_and_reads_no_order() {
    let orders = cbonsai_orders(CBONSAI_OUTDATED_STDOUT);

    let upgrade = steps(refused(), &outdated(&["cbonsai"]), &orders);

    assert_eq!(upgrade.actions, paru_upgrade());
    assert_eq!(orders.asked(), 0);
}

#[test]
fn a_route_the_helper_refuses_on_this_call_takes_paru() {
    let orders = cbonsai_orders(CBONSAI_OUTDATED_STDOUT);

    let upgrade = aur_steps(
        &system_with(accepted()),
        &outdated(&["cbonsai"]),
        &orders,
        Err(RouteProblem::NoAccount(1000)),
        1,
    );

    assert_eq!(upgrade.actions, paru_upgrade());
}

#[test]
fn an_outdated_list_that_is_not_read_takes_paru() {
    let orders = cbonsai_orders(CBONSAI_OUTDATED_STDOUT);
    let unread = FakeAurUpdates::new(Err("paru could not be asked"));

    let upgrade = steps(accepted(), &unread, &orders);

    assert_eq!(upgrade.actions, paru_upgrade());
    assert_eq!(orders.asked(), 0);
}
