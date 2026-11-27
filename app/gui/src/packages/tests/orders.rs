//! The window's reads of build orders: off the frame, one at a time, and
//! landed into the queue through `land_orders`.

use std::sync::{Arc, mpsc};

use tokio::runtime::Runtime;
use toolbox_core::domains::{AurPackages, AurWish, InstalledPackages};
use toolbox_core::features::packages::aur::{AurNames, toggle};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;
use toolbox_services::testing::until;
use toolbox_state::aur::AurRoute;
use toolbox_state::packages::PackageGraph;
use toolbox_state::testing::aur::order::{AUR_SYNC_VOTE_STDOUT, MISSING_STDOUT, OCTOPI_GIT_STDOUT};
use toolbox_system::testing::aur_orders::{FakeAurOrders, GatedAurOrders};
use toolbox_system::{AurOrderSource, SampleRoute};

use super::{Aur, Log, Machine};
use crate::packages::{Packages, PackagesSetup};

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(std::collections::BTreeSet::from([name(
        "paru",
    )])));
    snapshot.insert(PackageGraph::default());
    snapshot.insert(AurRoute::Accepted);
    snapshot.insert(AurNames::default());
    snapshot
}

fn queue(changes: &mut ChangeSet, text: &str) {
    toggle(&name(text), Toggle::On, &snapshot(), changes).unwrap();
}

fn packages(runtime: &Runtime, orders: Arc<dyn AurOrderSource>) -> Packages {
    Packages::new(
        Some(PackagesSetup {
            runtime: runtime.handle().clone(),
            source: Arc::new(Machine::default()),
            log: Arc::new(Log),
            aur: Arc::new(Aur::default()),
            orders,
            route: Arc::new(SampleRoute),
        }),
        egui::Context::default(),
    )
}

/// Waits for the running read to land and applies it, as the frame does.
fn land(packages: &mut Packages, changes: &mut ChangeSet) {
    let snapshot = snapshot();
    until("the order read never landed", || {
        packages.take();
        let Some(landing) = packages.take_orders() else {
            return false;
        };

        packages.land_orders(changes, &snapshot, landing);
        true
    });
}

fn wish(changes: &ChangeSet, text: &str) -> Option<AurWish> {
    changes.wish::<AurPackages>(&name(text)).cloned()
}

fn run_of(changes: &ChangeSet, text: &str) -> String {
    match wish(changes, text) {
        Some(AurWish::Built(order)) => order.run.as_str().to_owned(),
        other => panic!("{text} is not built: {other:?}"),
    }
}

#[test]
fn a_queued_build_is_read_once_and_becomes_built() {
    let runtime = Runtime::new().unwrap();
    let orders = Arc::new(FakeAurOrders::new(vec![(
        "octopi-git",
        Ok(OCTOPI_GIT_STDOUT),
    )]));

    let mut packages = packages(&runtime, Arc::<FakeAurOrders>::clone(&orders));
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");

    assert!(packages.ask_orders(&changes, &snapshot(), false));
    assert!(!packages.ask_orders(&changes, &snapshot(), false));
    land(&mut packages, &mut changes);

    assert!(matches!(
        wish(&changes, "octopi-git"),
        Some(AurWish::Built(_))
    ));
    assert_eq!(orders.asked(), 1);
    assert!(!packages.reading_orders());
}

#[test]
fn asking_for_a_read_returns_before_paru_answers() {
    let runtime = Runtime::new().unwrap();
    let (release, gate) = mpsc::channel();
    let orders = Arc::new(GatedAurOrders::new(
        FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]),
        gate,
    ));

    let mut packages = packages(&runtime, orders);
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");

    assert!(packages.ask_orders(&changes, &snapshot(), false));
    assert!(
        packages.reading_orders(),
        "the frame goes on while paru runs"
    );

    release.send(()).unwrap();
    land(&mut packages, &mut changes);
    assert!(matches!(
        wish(&changes, "octopi-git"),
        Some(AurWish::Built(_))
    ));
}

#[test]
fn a_build_queued_during_a_read_is_built_with_the_next_read_under_one_run() {
    let runtime = Runtime::new().unwrap();
    let (release, gate) = mpsc::channel();
    let orders = Arc::new(GatedAurOrders::new(
        FakeAurOrders::new(vec![
            ("octopi-git", Ok(OCTOPI_GIT_STDOUT)),
            ("aur-sync-vote", Ok(AUR_SYNC_VOTE_STDOUT)),
        ]),
        gate,
    ));

    let mut packages = packages(&runtime, orders);
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");

    assert!(packages.ask_orders(&changes, &snapshot(), false));
    queue(&mut changes, "aur-sync-vote");
    assert!(!packages.ask_orders(&changes, &snapshot(), false));

    release.send(()).unwrap();
    land(&mut packages, &mut changes);
    assert!(packages.ask_orders(&changes, &snapshot(), false));

    release.send(()).unwrap();
    release.send(()).unwrap();
    land(&mut packages, &mut changes);

    assert_eq!(
        run_of(&changes, "octopi-git"),
        run_of(&changes, "aur-sync-vote")
    );
}

#[test]
fn a_missing_order_keeps_its_build_waiting_with_its_reason() {
    let runtime = Runtime::new().unwrap();
    let orders = Arc::new(FakeAurOrders::new(vec![(
        "no-such-package-toolbox",
        Ok(MISSING_STDOUT),
    )]));

    let mut packages = packages(&runtime, orders);
    let mut changes = ChangeSet::default();
    queue(&mut changes, "no-such-package-toolbox");

    assert!(packages.ask_orders(&changes, &snapshot(), false));
    land(&mut packages, &mut changes);

    assert!(matches!(
        wish(&changes, "no-such-package-toolbox"),
        Some(AurWish::Waiting)
    ));
    assert_eq!(packages.order_missing().len(), 1);
    assert!(!packages.ask_orders(&changes, &snapshot(), false));
}

#[test]
fn a_build_removed_while_its_read_runs_is_not_put_back() {
    let runtime = Runtime::new().unwrap();
    let (release, gate) = mpsc::channel();
    let orders = Arc::new(GatedAurOrders::new(
        FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]),
        gate,
    ));

    let mut packages = packages(&runtime, orders);
    let mut changes = ChangeSet::default();
    queue(&mut changes, "octopi-git");

    assert!(packages.ask_orders(&changes, &snapshot(), false));
    toggle(&name("octopi-git"), Toggle::Off, &snapshot(), &mut changes).unwrap();

    release.send(()).unwrap();
    land(&mut packages, &mut changes);
    assert_eq!(wish(&changes, "octopi-git"), None);
}

#[test]
fn no_read_starts_without_a_queued_build() {
    let runtime = Runtime::new().unwrap();
    let orders = Arc::new(FakeAurOrders::new(Vec::new()));
    let mut packages = packages(&runtime, Arc::<FakeAurOrders>::clone(&orders));

    assert!(!packages.ask_orders(&ChangeSet::default(), &snapshot(), false));
    assert!(!packages.ask_orders(&ChangeSet::default(), &snapshot(), true));
    assert_eq!(orders.asked(), 0);
}
