//! Where the keyboard stays in the Console's header while its controls come
//! and go with the run.

use std::time::Instant;

use egui_kittest::kittest::Queryable as _;
use toolbox_gui::testing::PATIENCE;

use super::settle::{Gate, wait_for};
use super::{app, succeeding_script};

#[test]
fn a_focused_cancel_hands_the_keyboard_to_close_when_the_run_ends() {
    let gate = Gate::new();
    let (mut harness, _runtime) = app(gate.hold(&succeeding_script()));

    harness.get_by_label("Apply").click();
    let start = Instant::now();
    while harness.query_by_label("Cancel").is_none() {
        assert!(start.elapsed() < PATIENCE, "the run never showed");
        harness.step();
    }

    harness.get_by_label("Cancel").focus();
    harness.run();
    assert!(harness.get_by_label("Cancel").is_focused());

    gate.open();
    wait_for(&mut harness, "All changes applied");

    assert!(
        harness.get_by_label("Hide the Console").is_focused(),
        "the keyboard went elsewhere once Cancel was gone"
    );
    assert!(!harness.get_by_label("Console output actions").is_focused());
}

#[test]
fn close_keeps_the_focus_while_the_menu_appears_beside_it() {
    let gate = Gate::new();
    let (mut harness, _runtime) = app(gate.hold(&succeeding_script()));

    harness.get_by_label("Apply").click();
    wait_for(&mut harness, "Hide the Console");
    harness.get_by_label("Hide the Console").focus();
    harness.run();

    gate.open();
    wait_for(&mut harness, "All changes applied");

    assert!(harness.get_by_label("Hide the Console").is_focused());
}
