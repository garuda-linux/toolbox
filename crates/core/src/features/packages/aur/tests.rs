mod catalog;
mod restored;

use std::collections::BTreeSet;

use catalog::{aur, catalog};
use toolbox_plan::BuildRun;
use toolbox_state::aur::order::{AurOrder, parse_order};
use toolbox_state::aur::{AurIndex, AurRoute, RouteKind};
use toolbox_state::packages::PackageGraph;
use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;

use super::{AurCatalog, AurNames, give_order, needing_orders, route_refusal, row_state, toggle};
use crate::domains::{AurPackages, AurWish, BuildOrder, InstalledPackages, Packages};
use crate::features::packages::list::RowState;
use crate::features::packages::queue::Refusal;
use crate::features::search::ranked::Rank;
use crate::test_support::{package, packages};
use crate::{ChangeSet, Snapshot, Toggle};

#[test]
fn the_refusal_of_the_route_names_its_kind_and_an_accepted_route_none() {
    let mut snapshot = Snapshot::default();
    assert_eq!(route_refusal(&snapshot), None);

    snapshot.insert(AurRoute::Accepted);
    assert_eq!(route_refusal(&snapshot), None);

    snapshot.insert(AurRoute::Refused {
        kind: RouteKind::Shared,
        detail: String::new(),
    });
    assert_eq!(route_refusal(&snapshot), Some(RouteKind::Shared));
}

fn refused() -> AurRoute {
    AurRoute::Refused {
        kind: RouteKind::Linked,
        detail: "a folder on the way is missing".to_owned(),
    }
}

/// A snapshot with the installed packages, the given route answer and the
/// AUR's names, as the window has them once the copy is read.
fn with_route(names: &[&str], route: Option<AurRoute>, copy: AurNames) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(packages(names)));
    snapshot.insert(PackageGraph::default());
    snapshot.insert(copy);
    if let Some(route) = route {
        snapshot.insert(route);
    }

    snapshot
}

/// The paru path's snapshot: the helper refuses the route.
fn installed(names: &[&str]) -> Snapshot {
    with_route(names, Some(refused()), AurNames::default())
}

/// The split path's snapshot: the helper accepts the route.
fn split(names: &[&str]) -> Snapshot {
    with_route(names, Some(AurRoute::Accepted), AurNames::default())
}

/// The AUR's names with `conflicts` for `package`.
fn copy_with_conflicts(package_name: &str, conflicts: &[&str]) -> AurNames {
    AurNames {
        names: BTreeSet::new(),
        conflicts: [(
            package(package_name),
            conflicts.iter().map(|name| package(name)).collect(),
        )]
        .into(),
    }
}

fn octopi() -> AurOrder {
    parse_order(OCTOPI_GIT_STDOUT).unwrap()
}

fn built_octopi() -> BuildOrder {
    BuildOrder {
        order: octopi(),
        run: BuildRun::new("1").unwrap(),
        installed: BTreeSet::new(),
    }
}

#[test]
fn install_queues_an_aur_build_and_a_second_click_takes_it_back() {
    let snapshot = installed(&["paru"]);
    let mut changes = ChangeSet::default();
    let yay = package("yay-bin");

    toggle(&yay, Toggle::On, &snapshot, &mut changes).unwrap();
    assert_eq!(changes.wish::<AurPackages>(&yay), Some(&AurWish::Paru));
    assert_eq!(
        row_state(&yay, &snapshot, &changes),
        RowState {
            installed: false,
            queued: Some(Toggle::On),
        }
    );

    toggle(&yay, Toggle::Off, &snapshot, &mut changes).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn remove_of_an_installed_aur_package_is_a_package_removal() {
    let snapshot = installed(&["paru"]);
    let mut changes = ChangeSet::default();
    let paru = package("paru");

    toggle(&paru, Toggle::Off, &snapshot, &mut changes).unwrap();
    assert_eq!(changes.wish::<Packages>(&paru), Some(&Toggle::Off));
    assert_eq!(
        row_state(&paru, &snapshot, &changes).queued,
        Some(Toggle::Off)
    );

    toggle(&paru, Toggle::On, &snapshot, &mut changes).unwrap();
    assert!(changes.is_empty());
}

#[test]
fn nothing_is_queued_before_the_installed_packages_are_known() {
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(
            &package("yay-bin"),
            Toggle::On,
            &Snapshot::default(),
            &mut changes
        ),
        Err(Refusal::Unread.of(&package("yay-bin")))
    );
}

#[test]
fn ranked_packages_say_how_they_fit_the_most_popular_first() {
    let catalog = catalog();
    let needle = crate::features::search::Needle::new("paru");
    let ranked: Vec<(Rank, String)> = catalog
        .ranked(&needle)
        .map(|(rank, place)| (rank, catalog.entries()[place].name.to_string()))
        .collect();

    assert_eq!(
        ranked,
        [
            (Rank::InText, "aurutils".to_owned()),
            (Rank::Prefix, "paru-bin".to_owned()),
            (Rank::Exact, "paru".to_owned()),
        ]
    );
}

#[test]
fn an_accepted_route_queues_the_split_path_waiting_for_its_order() {
    let snapshot = split(&["paru"]);
    let mut changes = ChangeSet::default();
    let yay = package("yay-bin");

    toggle(&yay, Toggle::On, &snapshot, &mut changes).unwrap();

    assert_eq!(changes.wish::<AurPackages>(&yay), Some(&AurWish::Waiting));
}

#[test]
fn a_refused_route_queues_the_paru_path() {
    let snapshot = installed(&["paru"]);
    let mut changes = ChangeSet::default();
    let yay = package("yay-bin");

    toggle(&yay, Toggle::On, &snapshot, &mut changes).unwrap();

    assert_eq!(changes.wish::<AurPackages>(&yay), Some(&AurWish::Paru));
}

#[test]
fn no_route_answer_yet_refuses_as_unread() {
    let snapshot = with_route(&["paru"], None, AurNames::default());
    let mut changes = ChangeSet::default();
    let yay = package("yay-bin");

    assert_eq!(
        toggle(&yay, Toggle::On, &snapshot, &mut changes),
        Err(Refusal::Unread.of(&yay))
    );
    assert!(changes.is_empty());
}

#[test]
fn the_split_path_needs_the_aur_names_to_check_conflicts() {
    let snapshot = split(&["paru"]);
    let mut changes = ChangeSet::default();
    let yay = package("yay-bin");
    let without_copy = {
        let mut without = Snapshot::default();
        without.insert(InstalledPackages(packages(&["paru"])));
        without.insert(AurRoute::Accepted);
        without
    };

    assert_eq!(
        toggle(&yay, Toggle::On, &without_copy, &mut changes),
        Err(Refusal::Unread.of(&yay))
    );
    toggle(&yay, Toggle::On, &snapshot, &mut changes).unwrap();
}

#[test]
fn a_conflict_with_an_installed_package_refuses_the_split_path_and_names_it() {
    let snapshot = with_route(
        &["yay"],
        Some(AurRoute::Accepted),
        copy_with_conflicts("yay-bin", &["yay"]),
    );

    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle(&package("yay-bin"), Toggle::On, &snapshot, &mut changes),
        Err(Refusal::Conflicts.of(&package("yay")))
    );

    assert!(changes.is_empty());
}

#[test]
fn a_conflict_with_a_version_constraint_is_judged_by_the_name_only() {
    let mut yay_bin = aur("yay-bin", "Yet another yogurt, prebuilt", 4.0);
    yay_bin.conflicts = vec![toolbox_text::plain("yay<2")];
    let copy = AurCatalog::new(AurIndex {
        packages: vec![yay_bin],
        downloaded: 1_700_000_000,
        stale: false,
    })
    .names();

    let snapshot = with_route(&["yay"], Some(AurRoute::Accepted), copy);

    assert_eq!(
        toggle(
            &package("yay-bin"),
            Toggle::On,
            &snapshot,
            &mut ChangeSet::default()
        ),
        Err(Refusal::Conflicts.of(&package("yay")))
    );
}

#[test]
fn a_conflict_with_a_package_that_is_not_installed_refuses_nothing() {
    let snapshot = with_route(
        &["paru"],
        Some(AurRoute::Accepted),
        copy_with_conflicts("yay-bin", &["yay"]),
    );

    let mut changes = ChangeSet::default();

    toggle(&package("yay-bin"), Toggle::On, &snapshot, &mut changes).unwrap();

    assert_eq!(
        changes.wish::<AurPackages>(&package("yay-bin")),
        Some(&AurWish::Waiting)
    );
}

#[test]
fn the_paru_path_does_not_check_conflicts() {
    let snapshot = with_route(
        &["yay"],
        Some(refused()),
        copy_with_conflicts("yay-bin", &["yay"]),
    );

    let mut changes = ChangeSet::default();

    toggle(&package("yay-bin"), Toggle::On, &snapshot, &mut changes).unwrap();

    assert_eq!(
        changes.wish::<AurPackages>(&package("yay-bin")),
        Some(&AurWish::Paru)
    );
}

#[test]
fn a_second_click_takes_back_each_queued_wish_state() {
    let snapshot = split(&["paru"]);
    let octopi_git = package("octopi-git");
    let yay = package("yay-bin");
    let mut changes = ChangeSet::default();

    toggle(&yay, Toggle::On, &snapshot, &mut changes).unwrap();
    toggle(&octopi_git, Toggle::On, &snapshot, &mut changes).unwrap();
    give_order(&mut changes, &snapshot, &octopi_git, Some(built_octopi()));
    assert!(matches!(
        changes.wish::<AurPackages>(&octopi_git),
        Some(AurWish::Built(_))
    ));

    toggle(&yay, Toggle::Off, &snapshot, &mut changes).unwrap();
    toggle(&octopi_git, Toggle::Off, &snapshot, &mut changes).unwrap();

    assert!(changes.is_empty());
}

#[test]
fn the_builds_that_need_an_order_are_the_waiting_and_the_built_ones() {
    let snapshot = split(&["paru"]);
    let mut changes = ChangeSet::default();

    toggle(&package("yay-bin"), Toggle::On, &snapshot, &mut changes).unwrap();
    toggle(&package("octopi-git"), Toggle::On, &snapshot, &mut changes).unwrap();
    give_order(
        &mut changes,
        &snapshot,
        &package("octopi-git"),
        Some(built_octopi()),
    );
    changes.want::<AurPackages>(package("paru-bin"), AurWish::Paru, &snapshot);

    assert_eq!(
        needing_orders(&changes),
        [package("octopi-git"), package("yay-bin")]
    );
}

#[test]
fn an_order_makes_a_waiting_build_built_and_a_missing_one_keeps_it_waiting() {
    let snapshot = split(&["paru"]);
    let mut changes = ChangeSet::default();
    let octopi_git = package("octopi-git");
    toggle(&octopi_git, Toggle::On, &snapshot, &mut changes).unwrap();

    give_order(&mut changes, &snapshot, &octopi_git, Some(built_octopi()));
    assert_eq!(
        changes.wish::<AurPackages>(&octopi_git),
        Some(&AurWish::Built(built_octopi()))
    );

    give_order(&mut changes, &snapshot, &octopi_git, None);
    assert_eq!(
        changes.wish::<AurPackages>(&octopi_git),
        Some(&AurWish::Waiting)
    );
}

#[test]
fn an_order_never_changes_a_paru_build() {
    let snapshot = installed(&["paru"]);
    let mut changes = ChangeSet::default();
    let octopi_git = package("octopi-git");
    toggle(&octopi_git, Toggle::On, &snapshot, &mut changes).unwrap();

    give_order(&mut changes, &snapshot, &octopi_git, Some(built_octopi()));
    give_order(&mut changes, &snapshot, &octopi_git, None);

    assert_eq!(
        changes.wish::<AurPackages>(&octopi_git),
        Some(&AurWish::Paru)
    );
}
