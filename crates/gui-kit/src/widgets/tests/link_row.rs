use egui::Shape;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::super::link_row_at_least;
use crate::testing::{harness, tab_into};
use crate::{icons, tokens};

#[test]
fn a_link_row_is_a_link_named_by_its_title_and_reports_the_click() {
    let mut harness = harness(300.0, 120.0).build_ui_state(
        |ui, clicked: &mut bool| {
            if link_row_at_least(ui, icons::GLOBE, "Forum", Role::Link, 0.0).clicked() {
                *clicked = true;
            }
        },
        false,
    );

    harness.run();
    harness.get_by_role_and_label(Role::Link, "Forum").click();
    harness.run();

    assert!(*harness.state());
}

#[test]
fn a_row_that_acts_here_is_a_button() {
    let harness = harness(300.0, 120.0).build_ui(|ui| {
        link_row_at_least(ui, icons::GLOBE, "Call the exorcist", Role::Button, 0.0);
    });

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Call the exorcist")
            .is_some()
    );
}

/// Darkly's focus on a list item is a 1 point line under the row, with no
/// ring around it, and it shows only once a key moved the focus there.
#[test]
fn a_focused_link_row_shows_a_line_under_it_and_no_ring() {
    let mut harness = harness(300.0, 120.0).build_ui(|ui| {
        link_row_at_least(ui, icons::GLOBE, "Forum", Role::Link, 0.0);
    });

    harness.run();
    assert!(tab_into(&mut harness, Role::Link));
    harness.run();

    let row = harness.get_by_role_and_label(Role::Link, "Forum").rect();
    let tokens = tokens::current(&harness.ctx);
    let shapes: Vec<Shape> = harness
        .output()
        .shapes
        .iter()
        .map(|clipped| clipped.shape.clone())
        .collect();

    let line = shapes.iter().any(|shape| {
        matches!(shape, Shape::LineSegment { points, stroke }
            if stroke.color == tokens.accent_tint
                && (points[0].y - (row.bottom() - 0.5)).abs() < 0.5)
    });

    let ring = shapes
        .iter()
        .any(|shape| matches!(shape, Shape::Rect(rect) if rect.stroke.color == tokens.accent_ring));

    assert!(line, "no focus line under {row:?}");
    assert!(!ring, "a focus ring is drawn");
}
