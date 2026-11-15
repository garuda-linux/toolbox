use egui_kittest::kittest::Queryable as _;

use super::super::{menu_button, menu_item};
use crate::testing::harness;

#[test]
fn a_menu_item_picked_without_the_pointer_closes_the_menu() {
    let mut harness = harness(300.0, 300.0).build_ui_state(
        |ui, picked: &mut bool| {
            menu_button(ui, "More actions", |ui| {
                *picked |= menu_item(ui, "Copy").clicked();
            });
        },
        false,
    );

    harness.run();
    harness.get_by_label("More actions").click_accesskit();
    harness.run();
    harness.get_by_label("Copy").click_accesskit();
    harness.run();

    assert!(*harness.state());
    assert!(
        harness.query_by_label("Copy").is_none(),
        "the menu stayed open"
    );
}

/// The entry goes with the menu, so a dialog it opens would have no
/// opener to give the focus back to.
#[test]
fn a_picked_menu_item_hands_the_focus_to_the_menu_button() {
    let mut harness = harness(300.0, 300.0).build_ui(|ui| {
        menu_button(ui, "More actions", |ui| {
            drop(menu_item(ui, "Copy"));
        });
    });

    harness.run();
    harness.get_by_label("More actions").click_accesskit();
    harness.run();
    harness.get_by_label("Copy").click_accesskit();
    harness.run();

    assert!(harness.get_by_label("More actions").is_focused());
}
