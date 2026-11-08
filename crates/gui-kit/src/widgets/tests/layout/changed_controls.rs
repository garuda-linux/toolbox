//! Controls that change their width between frames, as a button that
//! turns into a different one after a click.

use egui::{Rect, vec2};

use super::super::super::split_row;
use crate::testing::harness;

/// The controls' widths in turn, and where the last ones were drawn.
struct Row {
    width: f32,
    controls: Rect,
    right: f32,
}

#[test]
fn controls_that_grow_never_reach_past_the_row_for_a_frame() {
    let mut harness = harness(600.0, 200.0).build_ui_state(
        |ui, row: &mut Row| {
            row.right = ui.max_rect().right();
            let ((), controls) = split_row(
                ui,
                220.0,
                |ui| {
                    ui.label("Public address");
                },
                |ui| {
                    ui.allocate_exact_size(vec2(row.width, 30.0), egui::Sense::click())
                        .0
                },
            );

            row.controls = controls;
        },
        Row {
            width: 0.0,
            controls: Rect::NOTHING,
            right: 0.0,
        },
    );

    harness.run();
    harness.state_mut().width = 180.0;
    harness.step();

    let row = harness.state();
    assert!(
        row.controls.right() <= row.right + 0.5,
        "the controls end at {}, the row at {}",
        row.controls.right(),
        row.right
    );
}
