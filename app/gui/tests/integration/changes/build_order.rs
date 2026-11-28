use std::collections::BTreeSet;
use std::sync::{Arc, mpsc};

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use egui_kittest::{Harness, HarnessBuilder};
use tokio::runtime::Runtime;
use tokio::sync::watch;
use toolbox_core::domains::{AurPackages, AurWish, InstalledPackages};
use toolbox_core::features::packages::aur::{AurNames, toggle};
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_gui::desktop::DesktopLook;
use toolbox_gui::packages::{PackagesSetup, SampleAur};
use toolbox_gui::{App, Setup};
use toolbox_plan::PackageName;
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_state::packages::PackageGraph;
use toolbox_state::testing::aur::order::{MISSING_STDOUT, OCTOPI_GIT_STDOUT};
use toolbox_system::testing::aur_orders::{FakeAurOrders, GatedAurOrders};
use toolbox_system::{AurOrderSource, Libalpm, PacmanLog, SampleRoute};

use super::{queued, review};
use crate::common::{HEIGHT, launch, launch_in, window};
use crate::system::step_until;

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn snapshot(route: AurRoute) -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([name("paru")])));
    snapshot.insert(PackageGraph::default());
    snapshot.insert(route);
    snapshot.insert(AurNames::default());
    snapshot
}

/// The queue with `text` queued as the route of `route` queues it.
pub(super) fn queued_build(text: &str, route: AurRoute) -> ChangeSet {
    let mut changes = ChangeSet::default();
    toggle(&name(text), Toggle::On, &snapshot(route), &mut changes).unwrap();
    changes
}

/// The app with `changes` queued and their orders answered by `orders`, in
/// the desktop look `look` when a guide picture asks for it.
pub(super) fn with_orders_in(
    builder: HarnessBuilder<App>,
    runtime: &Runtime,
    changes: ChangeSet,
    orders: Arc<dyn AurOrderSource>,
    look: Option<watch::Receiver<DesktopLook>>,
) -> Harness<'static, App> {
    let handle = runtime.handle().clone();
    launch_in(
        builder,
        Setup {
            look,
            queue: toolbox_gui::queue::Queue::in_memory(changes),
            packages: Some(PackagesSetup {
                runtime: handle,
                source: Arc::new(Libalpm::system()),
                log: Arc::new(PacmanLog::system()),
                aur: Arc::new(SampleAur),
                orders,
                route: Arc::new(SampleRoute),
            }),
            ..Setup::default()
        },
    )
}

fn with_orders(
    runtime: &Runtime,
    changes: ChangeSet,
    orders: Arc<dyn AurOrderSource>,
) -> Harness<'static, App> {
    with_orders_in(window(HEIGHT), runtime, changes, orders, None)
}

/// A queued AUR build has no build order yet, so Apply holds and says why.
#[test]
fn apply_waits_for_the_build_order_of_a_queued_aur_build() {
    let mut changes = ChangeSet::default();
    changes.want::<AurPackages>(
        "yay-bin".parse().unwrap(),
        AurWish::Waiting,
        &Snapshot::default(),
    );
    let harness = launch(queued(changes));

    let apply = harness.get_by_label("Apply");
    assert!(apply.accesskit_node().is_disabled());
    assert_eq!(
        apply.accesskit_node().description().as_deref(),
        Some("Apply waits for a build order. yay-bin waits for its build order.")
    );
}

/// The row of a build whose read has not come back says so, and Apply names it.
#[test]
fn a_build_whose_order_is_being_read_says_so() {
    let runtime = Runtime::new().unwrap();
    let (release, gate) = mpsc::channel::<()>();
    let orders = Arc::new(GatedAurOrders::new(
        FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]),
        gate,
    ));

    let mut harness = with_orders(
        &runtime,
        queued_build("octopi-git", AurRoute::Accepted),
        orders,
    );

    harness.run();
    review(&mut harness);

    assert!(harness.query_by_label("Reading the build order").is_some());
    let apply = harness.get_by_label("Apply the change");
    assert_eq!(
        apply.accesskit_node().description().as_deref(),
        Some("Apply waits until the build orders of octopi-git are read.")
    );
    drop(release);
}

/// A build paru cannot order names its package and the reason, and Read
/// again reads the orders once more.
#[test]
fn a_build_without_its_order_names_it_and_read_again_reads_once_more() {
    let runtime = Runtime::new().unwrap();
    let orders = Arc::new(FakeAurOrders::new(vec![(
        "no-such-package-toolbox",
        Ok(MISSING_STDOUT),
    )]));

    let mut harness = with_orders(
        &runtime,
        queued_build("no-such-package-toolbox", AurRoute::Accepted),
        Arc::<FakeAurOrders>::clone(&orders),
    );

    harness.run();
    review(&mut harness);
    step_until(&mut harness, "the reason", |harness| {
        harness
            .query_all_by_label_contains("has no build order: the AUR does not have it.")
            .next()
            .is_some()
    });

    let apply = harness.get_by_label("Apply the change");
    assert_eq!(
        apply.accesskit_node().description().as_deref(),
        Some(
            "Apply waits for a build order. no-such-package-toolbox has no build order: the AUR does not have it."
        )
    );

    harness
        .get_by_role_and_label(Role::Button, "Read Again")
        .click();
    harness.run();
    step_until(&mut harness, "the second read", |_| orders.asked() == 2);
}

/// A build that paru orders shows what it builds and every package it pulls
/// in, and the list folds open with all of them.
#[test]
fn a_built_build_lists_every_package_it_pulls_in() {
    let runtime = Runtime::new().unwrap();
    let orders = Arc::new(FakeAurOrders::new(vec![(
        "octopi-git",
        Ok(OCTOPI_GIT_STDOUT),
    )]));

    let mut harness = with_orders(
        &runtime,
        queued_build("octopi-git", AurRoute::Accepted),
        orders,
    );

    harness.run();
    review(&mut harness);
    step_until(&mut harness, "the build", |harness| {
        harness
            .query_by_label_contains("Builds alpm_octopi_utils-git, octopi-git.")
            .is_some()
    });

    assert!(
        harness.query_by_label("vala").is_none(),
        "the list is folded"
    );
    harness
        .get_by_role_and_label(Role::Button, "Packages 60")
        .click();
    harness.run();
    assert!(
        harness.query_by_label("vala").is_some(),
        "the last package is listed"
    );
    assert!(harness.query_by_label("qt6-base").is_some());
}

/// A setup whose route paru takes says why the build goes paru's own way.
#[test]
fn a_build_on_the_old_path_says_why_paru_asks_again() {
    let changes = queued_build(
        "yay-bin",
        AurRoute::Refused {
            kind: RouteKind::Shared,
            detail: String::new(),
        },
    );

    let mut harness = launch(queued(changes));
    review(&mut harness);

    assert!(
        harness
            .query_by_label_contains("This setup uses the old paru path")
            .is_some()
    );
    assert!(
        harness
            .query_by_label_contains("paru asks for the password for each step")
            .is_some()
    );
}
