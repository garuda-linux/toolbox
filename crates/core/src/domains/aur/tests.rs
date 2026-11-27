use std::collections::BTreeSet;

use toolbox_plan::{
    BuildRun, InstallReason, PackageName, PlannedAction, RootAction, TargetList, UserAction,
};
use toolbox_state::aur::order::{AurOrder, parse_order};
use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;
use toolbox_text::plain;

use super::{AurPackages, AurWish, BuildOrder, waiting_orders};
use crate::domain::{Domain as _, Stage, Staged};
use crate::domains::InstalledPackages;
use crate::features::packages::aur::build;
use crate::test_support::{package, packages};
use crate::{ChangeSet, Snapshot};

fn octopi() -> AurOrder {
    parse_order(OCTOPI_GIT_STDOUT).unwrap()
}

fn run(name: &str) -> BuildRun {
    BuildRun::new(name).unwrap()
}

fn built(order: &AurOrder, run_name: &str) -> AurWish {
    AurWish::Built(BuildOrder {
        order: order.clone(),
        run: run(run_name),
        installed: BTreeSet::new(),
    })
}

fn plan_of(changes: &[(&str, AurWish)]) -> Vec<Staged> {
    let pairs: Vec<(PackageName, AurWish)> = changes
        .iter()
        .map(|(name, wish)| (package(name), wish.clone()))
        .collect();

    let refs: Vec<(&PackageName, &AurWish)> =
        pairs.iter().map(|(item, wish)| (item, wish)).collect();

    AurPackages::plan(&refs)
}

/// What `build::plan` gives for the queued names, each at the domain's stage.
fn split_of(queued: &[&str], orders: &[AurOrder], run_name: &str) -> Vec<Staged> {
    let names: Vec<PackageName> = queued.iter().map(|name| package(name)).collect();

    build::plan(&names, orders, &run(run_name), &BTreeSet::new())
        .unwrap()
        .actions
        .into_iter()
        .map(|action| Staged::at(AurPackages::STAGE, action))
        .collect()
}

fn paru_install(names: &[&str]) -> Staged {
    let packages = TargetList::new(names.iter().map(|name| package(name))).unwrap();

    Staged::at(
        AurPackages::STAGE,
        PlannedAction::User(UserAction::InstallAur { packages }),
    )
}

#[test]
fn an_aur_install_is_done_once_the_package_is_installed() {
    let system = InstalledPackages(packages(&["yay-bin"]));

    assert!(AurPackages::is_satisfied(
        &package("yay-bin"),
        &AurWish::Paru,
        &system
    ));
    assert!(!AurPackages::is_satisfied(
        &package("paru-bin"),
        &AurWish::Paru,
        &system
    ));
}

#[test]
fn every_aur_install_goes_to_one_paru_run_as_the_user_after_the_repositories() {
    let yay = package("yay-bin");
    let paru = package("paru-bin");

    let planned = AurPackages::plan(&[(&yay, &AurWish::Paru), (&paru, &AurWish::Paru)]);

    assert_eq!(
        planned,
        [Staged::at(
            AurPackages::STAGE,
            PlannedAction::User(UserAction::InstallAur {
                packages: TargetList::new([paru, yay]).unwrap(),
            }),
        )]
    );
    assert!(AurPackages::STAGE > Stage::PACKAGES);
    assert!(AurPackages::STAGE < Stage::DKMS_BUILDS);
}

#[test]
fn an_aur_install_is_named_like_its_package() {
    let item = package("yay-bin");

    assert_eq!(AurPackages::key(&item), "yay-bin");
    assert_eq!(
        AurPackages::summary(&item, &AurWish::Paru).values,
        ["yay-bin"]
    );
}

#[test]
fn a_waiting_wish_plans_nothing() {
    assert_eq!(plan_of(&[("octopi-git", AurWish::Waiting)]), []);
}

#[test]
fn a_built_wish_plans_what_the_builder_plans_at_the_domain_stage() {
    let order = octopi();

    let planned = plan_of(&[("octopi-git", built(&order, "1a2b"))]);

    assert_eq!(planned, split_of(&["octopi-git"], &[order], "1a2b"));
}

#[test]
fn two_built_wishes_of_one_run_are_planned_together_from_their_orders() {
    let order = octopi();

    let planned = plan_of(&[
        ("alpm_octopi_utils-git", built(&order, "1a2b")),
        ("octopi-git", built(&order, "1a2b")),
    ]);

    assert_eq!(
        planned,
        split_of(
            &["alpm_octopi_utils-git", "octopi-git"],
            &[order.clone(), order],
            "1a2b"
        )
    );
}

#[test]
fn paru_and_built_wishes_plan_both_parts() {
    let order = octopi();

    let planned = plan_of(&[
        ("yay-bin", AurWish::Paru),
        ("octopi-git", built(&order, "1a2b")),
    ]);

    let mut expected = vec![paru_install(&["yay-bin"])];
    expected.extend(split_of(&["octopi-git"], &[order], "1a2b"));
    assert_eq!(planned, expected);
}

#[test]
fn a_waiting_wish_holds_back_the_built_wishes_and_not_the_paru_ones() {
    let order = octopi();

    let planned = plan_of(&[
        ("octopi-git", built(&order, "1a2b")),
        ("paru-bin", AurWish::Waiting),
        ("yay-bin", AurWish::Paru),
    ]);

    assert_eq!(planned, [paru_install(&["yay-bin"])]);
}

#[test]
fn built_wishes_of_two_runs_plan_no_split_actions() {
    let order = octopi();

    let planned = plan_of(&[
        ("octopi-git", built(&order, "1a2b")),
        ("alpm_octopi_utils-git", built(&order, "3c4d")),
    ]);

    assert_eq!(planned, []);
}

#[test]
fn a_built_order_with_a_missing_name_plans_no_split_actions() {
    let mut order = octopi();
    order.missing.push(plain("no-such-package"));

    let planned = plan_of(&[("octopi-git", built(&order, "1a2b"))]);

    assert_eq!(planned, []);
}

#[test]
fn a_built_wish_fits_only_an_item_its_order_names() {
    let wish = built(&octopi(), "1a2b");

    assert!(AurPackages::fits(&package("octopi-git"), &wish));
    assert!(!AurPackages::fits(&package("yay-bin"), &wish));
    assert!(AurPackages::fits(&package("yay-bin"), &AurWish::Paru));
    assert!(AurPackages::fits(&package("yay-bin"), &AurWish::Waiting));
}

#[test]
fn the_split_actions_keep_the_order_the_builder_gave_through_the_queue() {
    let order = octopi();
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(
        package("octopi-git"),
        built(&order, "1a2b"),
        &Snapshot::default(),
    );

    let actual: Vec<PlannedAction> = changes
        .actions()
        .into_iter()
        .map(|(_domain, action)| action)
        .collect();

    let expected: Vec<PlannedAction> = split_of(&["octopi-git"], &[order], "1a2b")
        .into_iter()
        .map(|step| step.action)
        .collect();

    assert_eq!(actual, expected);
}

#[test]
fn an_installed_name_of_the_order_is_kept_with_its_reason() {
    let order = octopi();
    let installed: BTreeSet<PackageName> =
        std::iter::once(package("alpm_octopi_utils-git")).collect();

    let wish = AurWish::Built(BuildOrder {
        order: order.clone(),
        run: run("1a2b"),
        installed: installed.clone(),
    });

    let planned = plan_of(&[("octopi-git", wish)]);

    let expected = build::plan(&[package("octopi-git")], &[order], &run("1a2b"), &installed)
        .unwrap()
        .actions;

    assert!(expected.iter().any(|action| matches!(
        action,
        PlannedAction::Root(RootAction::InstallBuilt {
            reason: InstallReason::Keep,
            packages: _,
            run: _,
        })
    )));
    assert_eq!(
        planned,
        expected
            .into_iter()
            .map(|action| Staged::at(AurPackages::STAGE, action))
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_waiting_wish_is_named_and_the_others_are_not() {
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(package("yay-bin"), AurWish::Waiting, &snapshot);
    changes.want::<AurPackages>(package("paru-bin"), AurWish::Paru, &snapshot);
    changes.want::<AurPackages>(package("octopi-git"), built(&octopi(), "1a2b"), &snapshot);

    assert_eq!(waiting_orders(&changes), [package("yay-bin")]);
}

#[test]
fn no_wish_waits_when_there_are_only_paru_and_built_wishes() {
    let snapshot = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(package("paru-bin"), AurWish::Paru, &snapshot);
    changes.want::<AurPackages>(package("octopi-git"), built(&octopi(), "1a2b"), &snapshot);

    assert_eq!(waiting_orders(&changes), []);
}
