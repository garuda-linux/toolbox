use std::collections::BTreeSet;

use toolbox_core::domains::{AurPackages, AurWish, InstalledPackages};
use toolbox_core::features::packages::aur::{AurNames, toggle};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;
use toolbox_state::aur::AurRoute;
use toolbox_state::packages::PackageGraph;
use toolbox_state::testing::aur::order::{AUR_SYNC_VOTE_STDOUT, MISSING_STDOUT, OCTOPI_GIT_STDOUT};
use toolbox_system::testing::aur_orders::FakeAurOrders;

use super::OrderReads;
use crate::aur_orders::{NoOrder, Orders, read_orders};

fn package(name: &str) -> PackageName {
    PackageName::new(name).unwrap()
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([package("paru")])));
    snapshot.insert(PackageGraph::default());
    snapshot.insert(AurRoute::Accepted);
    snapshot.insert(AurNames::default());
    snapshot
}

fn queue(changes: &mut ChangeSet, name: &str) {
    toggle(&package(name), Toggle::On, &snapshot(), changes).unwrap();
}

/// What `read_orders` answers for `names` of `source`, with one run.
fn orders_of(names: &[&str], source: &FakeAurOrders, run: u64) -> Orders {
    let names: Vec<PackageName> = names.iter().map(|name| package(name)).collect();
    read_orders(&names, source, &BTreeSet::new(), Ok(()), run).unwrap()
}

fn octopi() -> FakeAurOrders {
    FakeAurOrders::new(vec![
        ("octopi-git", Ok(OCTOPI_GIT_STDOUT)),
        ("aur-sync-vote", Ok(AUR_SYNC_VOTE_STDOUT)),
    ])
}

fn wish(changes: &ChangeSet, name: &str) -> Option<AurWish> {
    changes.wish::<AurPackages>(&package(name)).cloned()
}

#[test]
fn a_queued_build_starts_one_read_that_names_it() {
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");
    let mut reads = OrderReads::default();

    assert_eq!(
        reads.ask(&changes, false),
        Some(vec![package("octopi-git")])
    );
    assert!(reads.running());
    assert_eq!(reads.ask(&changes, false), None);
}

#[test]
fn the_order_lands_and_the_build_is_built() {
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");
    let snapshot = snapshot();
    let mut reads = OrderReads::default();
    reads.ask(&changes, false);

    reads.landed(
        &mut changes,
        &snapshot,
        orders_of(&["octopi-git"], &octopi(), 1),
    );

    assert!(!reads.running());
    assert!(matches!(
        wish(&changes, "octopi-git"),
        Some(AurWish::Built(_))
    ));
    assert_eq!(reads.ask(&changes, false), None);
}

#[test]
fn a_build_queued_during_a_read_is_read_with_the_others_under_one_run() {
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");
    let snapshot = snapshot();
    let mut reads = OrderReads::default();
    reads.ask(&changes, false);

    queue(&mut changes, "aur-sync-vote");
    assert_eq!(reads.ask(&changes, false), None, "one read at a time");

    reads.landed(
        &mut changes,
        &snapshot,
        orders_of(&["octopi-git"], &octopi(), 1),
    );
    assert_eq!(
        reads.ask(&changes, false),
        Some(vec![package("aur-sync-vote"), package("octopi-git")]),
        "the second read names every build, so both share its run"
    );

    reads.landed(
        &mut changes,
        &snapshot,
        orders_of(&["octopi-git", "aur-sync-vote"], &octopi(), 2),
    );

    let runs: Vec<String> = ["octopi-git", "aur-sync-vote"]
        .iter()
        .map(|name| match wish(&changes, name) {
            Some(AurWish::Built(order)) => order.run.as_str().to_owned(),
            other => panic!("{name} is not built: {other:?}"),
        })
        .collect();

    assert_eq!(runs, ["2", "2"]);
}

#[test]
fn a_missing_order_stays_waiting_with_its_reason_and_is_not_asked_again() {
    let source = FakeAurOrders::new(vec![("no-such-package-toolbox", Ok(MISSING_STDOUT))]);
    let mut changes = ChangeSet::default();
    queue(&mut changes, "no-such-package-toolbox");
    let snapshot = snapshot();
    let mut reads = OrderReads::default();
    reads.ask(&changes, false);

    reads.landed(
        &mut changes,
        &snapshot,
        orders_of(&["no-such-package-toolbox"], &source, 1),
    );

    assert!(matches!(
        wish(&changes, "no-such-package-toolbox"),
        Some(AurWish::Waiting)
    ));
    assert!(matches!(
        reads.missing(),
        [(name, NoOrder::Missing(_))] if *name == package("no-such-package-toolbox")
    ));
    assert_eq!(reads.ask(&changes, false), None);
}

#[test]
fn read_again_asks_for_every_build_even_one_already_read() {
    let source = FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]);
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");
    let snapshot = snapshot();
    let mut reads = OrderReads::default();
    reads.ask(&changes, false);
    reads.landed(
        &mut changes,
        &snapshot,
        orders_of(&["octopi-git"], &source, 1),
    );

    assert_eq!(reads.ask(&changes, true), Some(vec![package("octopi-git")]));
}

#[test]
fn an_apply_read_names_waiting_and_built_builds() {
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");
    let snapshot = snapshot();
    let mut reads = OrderReads::default();
    reads.ask(&changes, false);
    reads.landed(
        &mut changes,
        &snapshot,
        orders_of(&["octopi-git"], &octopi(), 1),
    );
    queue(&mut changes, "aur-sync-vote");

    assert_eq!(
        reads.ask(&changes, true),
        Some(vec![package("aur-sync-vote"), package("octopi-git")])
    );
}

#[test]
fn no_read_starts_without_a_queued_build() {
    let mut reads = OrderReads::default();
    assert_eq!(reads.ask(&ChangeSet::default(), false), None);
    assert_eq!(reads.ask(&ChangeSet::default(), true), None);
    assert!(!reads.running());
}

#[test]
fn a_package_removed_while_its_read_runs_is_not_put_back() {
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");
    let snapshot = snapshot();
    let mut reads = OrderReads::default();
    reads.ask(&changes, false);

    toggle(&package("octopi-git"), Toggle::Off, &snapshot, &mut changes).unwrap();
    reads.landed(
        &mut changes,
        &snapshot,
        orders_of(&["octopi-git"], &octopi(), 1),
    );

    assert_eq!(wish(&changes, "octopi-git"), None);
    assert!(reads.missing().is_empty());
}

#[test]
fn a_failed_start_frees_the_read_without_orders() {
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");
    let mut reads = OrderReads::default();
    reads.ask(&changes, false);

    reads.failed();

    assert!(!reads.running());
    assert_eq!(
        reads.ask(&changes, false),
        None,
        "the build is not asked again on its own"
    );
    assert_eq!(reads.ask(&changes, true), Some(vec![package("octopi-git")]));
}
