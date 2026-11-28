//! Home's AUR upgrade: the build orders of its packages are read off the
//! frame, and the plan that lands starts with the review.

use std::collections::BTreeSet;
use std::sync::{Arc, mpsc};

use tokio::runtime::Runtime;
use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::packages::aur::upgrade::{AurUpgrade, Start, start};
use toolbox_plan::{PlannedAction, UserAction};
use toolbox_services::testing::until;
use toolbox_state::aur::AurRoute;
use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;
use toolbox_system::testing::aur_orders::{FakeAurOrders, GatedAurOrders};

use super::orders::{name, packages};
use crate::packages::Packages;

/// A system where `octopi-git` is installed and the helper accepts the build
/// folder.
fn installed_octopi() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([name("octopi-git")])));
    snapshot.insert(AurRoute::Accepted);
    snapshot
}

fn fake(answer: Result<&'static str, &'static str>) -> FakeAurOrders {
    FakeAurOrders::new(vec![("octopi-git", answer)])
}

/// Asks Home's upgrade of `octopi-git` for its read.
fn request(packages: &mut Packages) {
    let outdated = [name("octopi-git")];
    let Start::Read(waiting) = start(&installed_octopi(), Some(&outdated)) else {
        panic!("the split path reads first");
    };

    packages.read_upgrade(waiting, BTreeSet::from([name("octopi-git")]));
}

fn landed(packages: &mut Packages) -> AurUpgrade {
    let mut landed = None;
    until("the upgrade never landed", || {
        packages.take();
        landed = packages.take_upgrade();
        landed.is_some()
    });

    landed.unwrap()
}

#[test]
fn the_split_upgrade_plans_the_review_first() {
    let runtime = Runtime::new().unwrap();
    let mut packages = packages(&runtime, Arc::new(fake(Ok(OCTOPI_GIT_STDOUT))));

    request(&mut packages);
    let plan = landed(&mut packages).after(Vec::new());

    assert!(matches!(
        plan.first(),
        Some(PlannedAction::User(UserAction::ReviewAur { bases: _ }))
    ));
    assert!(!packages.reading_upgrade());
}

#[test]
fn an_order_that_does_not_come_back_plans_paru() {
    let runtime = Runtime::new().unwrap();
    let mut packages = packages(&runtime, Arc::new(fake(Err("the AUR does not answer"))));

    request(&mut packages);
    let plan = landed(&mut packages).after(Vec::new());

    assert_eq!(plan, [PlannedAction::User(UserAction::UpgradeAur)]);
}

#[test]
fn the_read_is_busy_until_its_plan_lands() {
    let runtime = Runtime::new().unwrap();
    let (release, gate) = mpsc::channel();
    let orders = GatedAurOrders::new(fake(Ok(OCTOPI_GIT_STDOUT)), gate);
    let mut packages = packages(&runtime, Arc::new(orders));

    request(&mut packages);
    assert!(packages.reading_upgrade());

    release.send(()).unwrap();
    landed(&mut packages);
    assert!(!packages.reading_upgrade());
}

#[test]
fn without_a_runtime_paru_is_planned_at_once() {
    let mut packages = Packages::default();

    request(&mut packages);
    let plan = landed(&mut packages).after(Vec::new());

    assert_eq!(plan, [PlannedAction::User(UserAction::UpgradeAur)]);
}
