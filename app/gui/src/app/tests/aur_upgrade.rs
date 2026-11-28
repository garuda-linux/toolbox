//! Home's AUR button: a split upgrade reads its build orders off the frame
//! and runs once they landed, and every other click runs at once or not at
//! all.

use std::collections::BTreeSet;
use std::sync::{Arc, mpsc};

use egui_kittest::Harness;
use tokio::runtime::Runtime;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::maintenance::RunNow;
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::PackageName;
use toolbox_services::testing::until;
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;
use toolbox_system::testing::aur_orders::{FakeAurOrders, GatedAurOrders};
use toolbox_system::{AurOrderSource, PendingUpdate, Status};
use toolbox_text::plain;

use super::orders::app_with;
use crate::app::App;
use crate::command::HomeCommand;

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap()
}

fn update(text: &str) -> PendingUpdate {
    PendingUpdate {
        name: name(text),
        from: plain("1.0"),
        to: plain("1.1"),
    }
}

fn status(outdated: &[&str]) -> Status {
    let updates = outdated.iter().copied().map(update).collect();

    Status {
        updates: Ok(Vec::new()),
        aur_updates: Ok(updates),
        merges: Ok(Vec::new()),
        last_update: Ok(None),
        health: Ok(Vec::new()),
    }
}

/// An app on Home with `orders` as its build order source.
fn open<S>(runtime: &Runtime, orders: &Arc<S>) -> Harness<'static, App>
where
    S: AurOrderSource + 'static,
{
    let source: Arc<dyn AurOrderSource> = Arc::<S>::clone(orders);
    let mut harness = app_with(runtime, source, ChangeSet::default());
    harness.run();
    harness
}

fn fake(answers: Vec<(&'static str, Result<&'static str, &'static str>)>) -> Arc<FakeAurOrders> {
    Arc::new(FakeAurOrders::new(answers))
}

fn gated() -> (mpsc::Sender<()>, Arc<GatedAurOrders>) {
    let (release, gate) = mpsc::channel();
    let fake = FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]);

    (release, Arc::new(GatedAurOrders::new(fake, gate)))
}

/// The system the click reads: `octopi-git` installed, the route as given,
/// and the status check's outdated packages.
fn seeded(harness: &mut Harness<'static, App>, route: AurRoute, outdated: &[&str]) {
    let mut snapshot = Snapshot::default();
    snapshot.insert(InstalledPackages(BTreeSet::from([name("octopi-git")])));
    snapshot.insert(route);

    let app = harness.state_mut();
    app.seen = Arc::new(snapshot);
    app.services.home.set_status(status(outdated));
}

fn click(harness: &mut Harness<'static, App>) {
    harness
        .state_mut()
        .run_home(&HomeCommand::RunNow(RunNow::AurUpgrade));
}

/// Whether a plan of the click reached the Console.
fn plan_sent(harness: &Harness<'static, App>) -> bool {
    harness.state().state.console_open
}

#[test]
fn a_split_upgrade_runs_once_its_orders_are_read() {
    let runtime = Runtime::new().unwrap();
    let (release, orders) = gated();
    let mut harness = open(&runtime, &orders);

    seeded(&mut harness, AurRoute::Accepted, &["octopi-git"]);
    click(&mut harness);
    assert!(harness.state().services.packages.reading_upgrade());
    assert!(!plan_sent(&harness), "the plan waits for the orders");

    release.send(()).unwrap();
    until("the upgrade never ran", || {
        harness.step();
        plan_sent(&harness)
    });

    assert!(!harness.state().services.packages.reading_upgrade());
    assert_eq!(orders.asked(), 1);
}

#[test]
fn a_second_click_while_the_orders_are_read_starts_nothing() {
    let runtime = Runtime::new().unwrap();
    let (release, orders) = gated();
    let mut harness = open(&runtime, &orders);

    seeded(&mut harness, AurRoute::Accepted, &["octopi-git"]);
    click(&mut harness);
    click(&mut harness);

    release.send(()).unwrap();
    until("the upgrade never ran", || {
        harness.step();
        plan_sent(&harness)
    });

    assert_eq!(orders.asked(), 1);
}

#[test]
fn a_refused_route_runs_paru_at_once_without_a_read() {
    let runtime = Runtime::new().unwrap();
    let orders = fake(Vec::new());
    let mut harness = open(&runtime, &orders);

    let refused = AurRoute::Refused {
        kind: RouteKind::Other,
        detail: String::new(),
    };

    seeded(&mut harness, refused, &["octopi-git"]);
    click(&mut harness);

    assert!(plan_sent(&harness));
    assert_eq!(orders.asked(), 0);
}

#[test]
fn an_order_that_does_not_come_back_runs_paru() {
    let runtime = Runtime::new().unwrap();
    let orders = fake(vec![("octopi-git", Err("the AUR does not answer"))]);
    let mut harness = open(&runtime, &orders);

    seeded(&mut harness, AurRoute::Accepted, &["octopi-git"]);
    click(&mut harness);
    until("paru never ran", || {
        harness.step();
        plan_sent(&harness)
    });

    assert_eq!(orders.asked(), 1);
}

#[test]
fn nothing_outdated_runs_nothing() {
    let runtime = Runtime::new().unwrap();
    let orders = fake(Vec::new());
    let mut harness = open(&runtime, &orders);

    seeded(&mut harness, AurRoute::Accepted, &[]);
    click(&mut harness);

    assert!(!plan_sent(&harness));
    assert_eq!(orders.asked(), 0);
}
