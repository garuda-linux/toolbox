//! A waiting Apply is dropped by Cancel and by a closed popover.

use std::sync::{Arc, mpsc};

use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;
use toolbox_system::testing::aur_orders::{FakeAurOrders, GatedAurOrders};

use super::orders::{app_with, click_apply, plan_sent, queued, until_read_landed};
use crate::command::{AppCommand, Command, PackagesCommand};

/// A read of octopi-git that waits for the test to release it.
fn gated() -> (mpsc::Sender<()>, Arc<GatedAurOrders>) {
    let (release, gate) = mpsc::channel();
    let orders = Arc::new(GatedAurOrders::new(
        FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]),
        gate,
    ));

    (release, orders)
}

#[test]
fn cancel_drops_the_apply_that_waits_for_its_read() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (release, orders) = gated();
    let mut harness = app_with(&runtime, orders, queued(&["octopi-git"]));
    harness.run();

    click_apply(&mut harness);
    let ctx = harness.ctx.clone();
    harness.state_mut().run(
        &ctx,
        Command::App(AppCommand::Packages(PackagesCommand::CancelApply)),
    );

    release.send(()).unwrap();
    release.send(()).unwrap();
    until_read_landed(&mut harness);

    assert!(!plan_sent(&harness));
    assert_eq!(harness.state().services.packages.apply_revision(), None);
}

#[test]
fn closing_the_popover_drops_the_apply_that_waits_for_its_read() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (release, orders) = gated();
    let mut harness = app_with(&runtime, orders, queued(&["octopi-git"]));
    harness.run();

    harness.state_mut().changes_ui.open = true;
    click_apply(&mut harness);
    assert!(harness.state().services.packages.apply_revision().is_some());

    harness.state_mut().changes_ui.open = false;
    harness.state_mut().changes_ui.dismissed = true;
    harness.run();
    assert_eq!(harness.state().services.packages.apply_revision(), None);

    release.send(()).unwrap();
    release.send(()).unwrap();
    until_read_landed(&mut harness);
    assert!(!plan_sent(&harness));
}

#[test]
fn a_popover_that_stays_open_keeps_the_apply_waiting() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (release, orders) = gated();
    let mut harness = app_with(&runtime, orders, queued(&["octopi-git"]));
    harness.run();

    harness.state_mut().changes_ui.open = true;
    click_apply(&mut harness);
    harness.run();

    assert!(harness.state().services.packages.apply_revision().is_some());
    release.send(()).unwrap();
    release.send(()).unwrap();
}
