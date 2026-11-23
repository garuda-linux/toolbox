use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Toggle;
use toolbox_core::domains::ModuleBlacklist;
use toolbox_core::testing::Queue as _;

use super::{module, page, tab};

#[test]
fn clear_filters_brings_back_what_the_chips_hid() {
    let mut harness = page();
    tab(&mut harness, "Drivers");

    for (group, chip) in [("State", "Loaded"), ("Kind", "Wireless")] {
        harness
            .get_by_role_and_label(Role::Group, group)
            .get_by_label(chip)
            .click();
        harness.run();
    }

    assert!(harness.query_by_label("No driver matches").is_some());

    harness
        .get_by_role_and_label(Role::Button, "Clear Filters")
        .click();
    harness.run();

    assert!(harness.query_by_label("Showing 3 of 3 drivers").is_some());
}

#[test]
fn a_frame_with_the_same_inputs_builds_no_driver_rows() {
    let mut harness = page();
    tab(&mut harness, "Drivers");
    let built = harness.state().page.driver_rows.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.driver_rows.builds(), built);

    "e1000".clone_into(&mut harness.state_mut().page.drivers.search);
    harness.run();
    assert_eq!(harness.state().page.driver_rows.builds(), built + 1);

    let state = harness.state_mut();
    let snapshot = std::sync::Arc::clone(&state.fixture.snapshot);
    state
        .fixture
        .changes
        .want::<ModuleBlacklist>(module("e1000e"), Toggle::On, &snapshot);
    state.fixture.revision += 1;
    harness.run();
    assert_eq!(harness.state().page.driver_rows.builds(), built + 2);
}

#[test]
fn each_chip_row_has_its_name_before_it() {
    let mut harness = page();
    tab(&mut harness, "Drivers");

    for group in ["State", "Kind"] {
        // egui gives a group no bounds of its own, so its first chip stands in.
        let chips = harness
            .get_by_role_and_label(Role::Group, group)
            .get_by_label("All")
            .rect();

        let label = harness.get_by_role_and_label(Role::Label, group).rect();

        assert!(
            label.right() <= chips.left() && chips.y_range().contains(label.center().y),
            "{group} at {label:?} is not before its chips at {chips:?}"
        );
    }
}

#[test]
fn a_driver_kind_is_plain_text_without_a_dot() {
    let mut harness = page();
    tab(&mut harness, "Drivers");

    let kind = harness
        .query_all_by_label("Wireless")
        .find(|node| node.accesskit_node().role() == Role::Label)
        .expect("the kind of iwlwifi")
        .rect();

    let dot = harness.output().shapes.iter().any(|clipped| {
        matches!(&clipped.shape, egui::Shape::Circle(circle)
            if kind.y_range().contains(circle.center.y)
                && circle.center.x < kind.left()
                && circle.center.x > kind.left() - 12.0)
    });

    assert!(!dot, "a dot before the kind");
}
