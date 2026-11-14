use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
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
        .get_by_role_and_label(Role::Button, "Clear filters")
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
