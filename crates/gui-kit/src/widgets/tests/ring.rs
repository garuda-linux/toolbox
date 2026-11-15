use egui::epaint::RectShape;
use egui::{Color32, Rect, Shape};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{ButtonKind, card, card_row, pill_button};
use crate::testing::harness;
use crate::theme::CONTROL_RADIUS;
use crate::tokens;

fn stroked(harness: &Harness<'_>, colour: Color32) -> Vec<(RectShape, Rect)> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            if let Shape::Rect(rect) = &clipped.shape
                && rect.stroke.color == colour
                && rect.stroke.width > 0.0
            {
                Some((rect.clone(), clipped.clip_rect))
            } else {
                None
            }
        })
        .collect()
}

/// A Soft button in a card's row, focused and under the pointer.
fn focused_in_a_row() -> Harness<'static> {
    let mut harness = harness(400.0, 120.0).build_ui(|ui| {
        card(ui, |ui| {
            card_row(ui, true, |ui| pill_button(ui, "Edit…", ButtonKind::Soft));
        });
    });

    harness.run();
    harness.get_by_label("Edit…").focus();
    harness.run();
    harness.get_by_label("Edit…").hover();
    harness.run();
    harness
}

#[test]
fn a_focus_ring_in_a_row_is_round_and_whole() {
    let harness = focused_in_a_row();
    let ring = tokens::current(&harness.ctx).accent_ring;
    let button = harness.get_by_label("Edit…").rect();
    let found = stroked(&harness, ring);
    let [(shape, clip)] = found.as_slice() else {
        panic!("one ring is painted");
    };

    let outer = shape.rect.expand(shape.stroke.width);
    assert!(clip.contains_rect(outer), "{outer:?} is cut by {clip:?}");
    assert!(
        (outer.center() - button.center()).length() < 0.5,
        "ring {outer:?} around {button:?}"
    );

    // The ring keeps the button's corners at its distance from it.
    let radius = shape.corner_radius;
    let gap = (shape.rect.height() - button.height()) / 2.0;
    let expected = f32::from(CONTROL_RADIUS) + gap;
    assert!(
        radius.nw == radius.se && (f32::from(radius.nw) - expected).abs() < 0.5,
        "corners {radius:?} round a button with {CONTROL_RADIUS} corners {gap} away"
    );
}

#[test]
fn a_focused_button_drops_its_rim_for_the_ring() {
    let harness = focused_in_a_row();
    let rim = tokens::current(&harness.ctx).rim;
    let button = harness.get_by_label("Edit…").rect();
    let rimmed = stroked(&harness, rim)
        .into_iter()
        .any(|(shape, _)| shape.rect == button);

    assert!(!rimmed, "a rim and a ring make two outlines");
}
