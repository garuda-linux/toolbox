use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::super::removable_tag;
use crate::testing::{circle_fills, fill_under, harness};

fn shown() -> egui_kittest::Harness<'static, bool> {
    let mut harness = harness(400.0, 100.0).build_ui_state(
        |ui, removed: &mut bool| {
            if removable_tag(ui, "quiet", "Remove quiet") {
                *removed = true;
            }
        },
        false,
    );

    harness.run();
    harness
}

#[test]
fn the_remove_cross_sits_inside_its_tag_with_a_finger_sized_target() {
    let harness = shown();
    let tag = harness.get_by_role_and_label(Role::Label, "quiet").rect();
    let cross = harness
        .get_by_role_and_label(Role::Button, "Remove quiet")
        .rect();

    assert!(cross.width() >= 24.0 && cross.height() >= 24.0, "{cross:?}");
    assert_eq!(
        fill_under(&harness, egui::pos2(cross.left() + 4.0, cross.center().y)),
        fill_under(&harness, tag.center()),
        "the cross sits outside the tag's fill"
    );
}

#[test]
fn a_hovered_cross_shows_a_disc_off_the_tag_fill() {
    let mut harness = shown();
    let tag = harness.get_by_role_and_label(Role::Label, "quiet").rect();
    harness
        .get_by_role_and_label(Role::Button, "Remove quiet")
        .hover();
    harness.run();

    let fill = fill_under(&harness, tag.center());
    let discs = circle_fills(&harness);
    assert_eq!(discs.len(), 1, "{discs:?}");
    assert_ne!(discs.first().copied(), fill);
}

#[test]
fn the_cross_removes_the_tag() {
    let mut harness = shown();
    harness
        .get_by_role_and_label(Role::Button, "Remove quiet")
        .click();
    harness.run();

    assert!(*harness.state());
}
