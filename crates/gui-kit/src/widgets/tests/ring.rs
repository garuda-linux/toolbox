//! Darkly has no focus ring: a focused push button shows its focus as the
//! fill, in a row as anywhere else.

use egui::Shape;
use egui_kittest::kittest::Queryable as _;

use super::super::{ButtonKind, card, card_row, pill_button};
use crate::testing::harness;
use crate::tokens;

#[test]
fn a_focused_button_in_a_row_is_filled_after_a_key_and_draws_no_ring() {
    let mut harness = harness(400.0, 120.0).build_ui(|ui| {
        card(ui, |ui| {
            card_row(ui, true, |ui| pill_button(ui, "Edit…", ButtonKind::Soft));
        });
    });

    harness.run();
    harness.get_by_label("Edit…").focus();
    harness.run();

    // The program's focus shows no fill until a key moves it.
    let tokens = tokens::current(&harness.ctx);
    let filled_before = harness.output().shapes.iter().any(|clipped| {
        matches!(&clipped.shape, Shape::Rect(rect) if rect.fill == tokens.decoration_focus)
    });

    assert!(!filled_before, "no key yet, so no focus fill");

    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.run();

    let rings = harness
        .output()
        .shapes
        .iter()
        .filter(|clipped| {
            matches!(&clipped.shape, Shape::Rect(rect)
                if rect.stroke.color == tokens.accent_ring && rect.stroke.width > 0.0)
        })
        .count();

    assert_eq!(rings, 0, "Darkly draws no focus ring");

    let filled = harness.output().shapes.iter().any(|clipped| {
        matches!(&clipped.shape, Shape::Rect(rect) if rect.fill == tokens.decoration_focus)
    });

    assert!(filled, "the focus colour fills the button");
}
