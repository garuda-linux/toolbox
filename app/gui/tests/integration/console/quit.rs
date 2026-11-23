//! Quitting while a plan runs: the question, then a note until the
//! cancelled run ends.

use egui::{ViewportEvent, ViewportId};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui::App;

use super::settle::Gate;
use super::{JAVA, app, succeeding_script, wait_for};
use crate::system::step_until;

const STOPPING: &str = "Stopping the Run";

fn ask_to_close(harness: &mut Harness<'static, App>) {
    harness
        .input_mut()
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
}

#[test]
fn a_quit_confirmed_during_a_run_shows_a_stopping_note_until_the_run_ends() {
    let gate = Gate::new();
    let (mut harness, _runtime) = app(gate.hold(&succeeding_script()), JAVA);
    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "Hide the Console");

    ask_to_close(&mut harness);
    step_until(&mut harness, "the question", |harness| {
        harness.query_by_label("Cancel and Quit").is_some()
    });

    harness.get_by_label("Cancel and Quit").click();
    step_until(&mut harness, "the stopping note", |harness| {
        harness.query_all_by_label(STOPPING).next().is_some()
    });

    gate.open();
    step_until(&mut harness, "the run to end", |harness| {
        harness.query_all_by_label(STOPPING).next().is_none()
    });
}

#[test]
fn a_second_close_while_stopping_asks_nothing() {
    let gate = Gate::new();
    let (mut harness, _runtime) = app(gate.hold(&succeeding_script()), JAVA);
    harness.get_by_label("Apply").click();
    harness.step();
    wait_for(&mut harness, "Hide the Console");

    ask_to_close(&mut harness);
    step_until(&mut harness, "the question", |harness| {
        harness.query_by_label("Cancel and Quit").is_some()
    });
    harness.get_by_label("Cancel and Quit").click();
    harness.step();

    ask_to_close(&mut harness);
    harness.step();
    harness.step();
    assert!(
        harness.query_by_label("Cancel and Quit").is_none(),
        "the question came back"
    );
    gate.open();
}
