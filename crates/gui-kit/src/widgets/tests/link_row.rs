use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::super::link_row;
use crate::icons;
use crate::testing::harness;

#[test]
fn a_link_row_is_a_link_named_by_its_title_and_reports_the_click() {
    let mut harness = harness(300.0, 120.0).build_ui_state(
        |ui, clicked: &mut bool| {
            if link_row(ui, icons::GLOBE, "Forum", Role::Link).clicked() {
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
        link_row(ui, icons::GLOBE, "Call the exorcist", Role::Button);
    });

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Call the exorcist")
            .is_some()
    );
}
