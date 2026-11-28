//! Apply with queued AUR builds: a fresh read of their orders comes first,
//! and the plan is sent in the pass that the read lands in.

use std::collections::BTreeSet;
use std::sync::{Arc, mpsc};

use egui_kittest::Harness;
use tokio::runtime::Runtime;
use toolbox_core::domains::{AurPackages, AurWish, InstalledPackages};
use toolbox_core::features::packages::aur::{AurNames, toggle};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::PackageName;
use toolbox_services::testing::until;
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_state::packages::PackageGraph;
use toolbox_state::testing::aur::order::{MISSING_STDOUT, OCTOPI_GIT_STDOUT};
use toolbox_system::testing::aur_orders::{FakeAurOrders, GatedAurOrders};
use toolbox_system::{AurOrderSource, Libalpm, PacmanLog, SampleRoute};

use crate::app::{App, Setup};
use crate::command::{Command, UiCommand};
use crate::packages::{PackagesSetup, SampleAur};
use crate::queue::Queue;

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

/// The queue with a split build of each of `names`.
pub(super) fn queued(names: &[&str]) -> ChangeSet {
    let mut changes = ChangeSet::default();
    for text in names {
        toggle(
            &name(text),
            Toggle::On,
            &snapshot(AurRoute::Accepted),
            &mut changes,
        )
        .unwrap();
    }

    changes
}

pub(super) fn app_with(
    runtime: &Runtime,
    orders: Arc<dyn AurOrderSource>,
    changes: ChangeSet,
) -> Harness<'static, App> {
    let handle = runtime.handle().clone();
    Harness::builder().build_eframe(move |cc| {
        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(changes),
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
    })
}

pub(super) fn click_apply(harness: &mut Harness<'static, App>) {
    let ctx = harness.ctx.clone();
    harness.state_mut().run(&ctx, Command::Ui(UiCommand::Apply));
}

pub(super) fn run_of(harness: &Harness<'static, App>, text: &str) -> Option<String> {
    match harness
        .state()
        .queue
        .changes()
        .wish::<AurPackages>(&name(text))
    {
        Some(AurWish::Built(order)) => Some(order.run.as_str().to_owned()),
        _ => None,
    }
}

/// Frames until the read of build orders that runs has landed.
pub(super) fn until_read_landed(harness: &mut Harness<'static, App>) {
    until("the read never landed", || {
        harness.step();
        !harness.state().services.packages.reading_orders()
    });
}

pub(super) fn plan_sent(harness: &Harness<'static, App>) -> bool {
    harness.state().state.console_open
}

#[test]
fn apply_sends_the_plan_once_a_fresh_read_landed_with_a_new_run() {
    let runtime = Runtime::new().unwrap();
    let orders = Arc::new(FakeAurOrders::new(vec![(
        "octopi-git",
        Ok(OCTOPI_GIT_STDOUT),
    )]));

    let mut harness = app_with(
        &runtime,
        Arc::<FakeAurOrders>::clone(&orders),
        queued(&["octopi-git"]),
    );

    until("the first read never landed", || {
        harness.step();
        run_of(&harness, "octopi-git").is_some()
    });
    let first = run_of(&harness, "octopi-git");

    click_apply(&mut harness);
    assert!(!plan_sent(&harness), "the plan waits for the fresh read");

    until("the plan was never sent", || {
        harness.step();
        plan_sent(&harness)
    });

    assert_ne!(
        run_of(&harness, "octopi-git"),
        first,
        "the plan builds the new run"
    );
    assert_eq!(orders.asked(), 2);
}

#[test]
fn a_second_apply_while_the_read_runs_is_ignored() {
    let runtime = Runtime::new().unwrap();
    let (release, gate) = mpsc::channel();
    let orders = Arc::new(GatedAurOrders::new(
        FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]),
        gate,
    ));

    let mut harness = app_with(&runtime, orders, queued(&["octopi-git"]));
    harness.run();

    click_apply(&mut harness);
    click_apply(&mut harness);
    assert!(!plan_sent(&harness));

    // The read that the queue started itself, then the apply's own read.
    release.send(()).unwrap();
    release.send(()).unwrap();
    until("the plan was never sent", || {
        harness.step();
        plan_sent(&harness)
    });
    assert_eq!(harness.state().services.packages.apply_revision(), None);
}

#[test]
fn a_queue_edit_during_the_read_drops_the_apply() {
    let runtime = Runtime::new().unwrap();
    let (release, gate) = mpsc::channel();
    let orders = Arc::new(GatedAurOrders::new(
        FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]),
        gate,
    ));

    let mut harness = app_with(&runtime, orders, queued(&["octopi-git"]));
    harness.run();

    click_apply(&mut harness);
    harness.state_mut().queue.clear();

    release.send(()).unwrap();
    until("the read never landed", || {
        harness.step();
        !harness.state().services.packages.reading_orders()
    });

    assert!(!plan_sent(&harness));
    assert_eq!(harness.state().services.packages.apply_revision(), None);
}

#[test]
fn a_missing_order_holds_apply_and_sends_no_plan() {
    let runtime = Runtime::new().unwrap();
    let orders = Arc::new(FakeAurOrders::new(vec![(
        "no-such-package-toolbox",
        Ok(MISSING_STDOUT),
    )]));

    let mut harness = app_with(&runtime, orders, queued(&["no-such-package-toolbox"]));
    harness.run();

    click_apply(&mut harness);
    until("the read never landed", || {
        harness.step();
        !harness.state().services.packages.reading_orders()
            && harness.state().services.packages.apply_revision().is_none()
    });

    assert!(!plan_sent(&harness));
    assert_eq!(harness.state().apply_held(), Some("changes-wait-orders"));

    let ctx = harness.ctx.clone();
    let (hold, _notes) = harness.state().holds(&ctx);
    assert_eq!(
        hold.map(|hold| hold.text),
        Some("Apply waits for a build order. no-such-package-toolbox has no build order: the AUR does not have it.".to_owned())
    );
}

#[test]
fn a_queue_without_split_builds_is_sent_in_the_pass_of_the_click() {
    let runtime = Runtime::new().unwrap();
    let orders = Arc::new(FakeAurOrders::new(Vec::new()));
    let route = AurRoute::Refused {
        kind: RouteKind::Other,
        detail: String::new(),
    };

    let mut changes = ChangeSet::default();
    toggle(
        &name("paru-bin"),
        Toggle::On,
        &snapshot(route),
        &mut changes,
    )
    .unwrap();

    let mut harness = app_with(&runtime, Arc::<FakeAurOrders>::clone(&orders), changes);
    harness.run();
    click_apply(&mut harness);

    assert!(
        plan_sent(&harness),
        "no read stands between the click and the plan"
    );
    assert_eq!(orders.asked(), 0);
}
