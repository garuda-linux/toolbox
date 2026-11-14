use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Queued;
use toolbox_core::domains::{NewGroups, RemovedUsers};
use toolbox_core::features::users::listing::GroupOrder;
use toolbox_core::testing::Queue as _;
use toolbox_plan::HomeFolder;

use super::{group_name, name, page, unlocked};

#[test]
fn a_frame_with_the_same_inputs_builds_no_user_rows() {
    let mut harness = page(unlocked());
    let built = harness.state().page.users.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.users.builds(), built);
    assert!(harness.query_by_label("Ada Lovelace").is_some());

    harness
        .get_by_role_and_label(Role::Button, "Regular users only")
        .click();
    harness.run();
    assert_eq!(harness.state().page.users.builds(), built + 1);
    assert!(harness.query_by_label("http").is_some());

    harness.state_mut().fixture.revision += 1;
    harness.run();
    assert_eq!(harness.state().page.users.builds(), built + 2);
}

#[test]
fn return_on_a_row_queued_for_removal_opens_no_edit() {
    let mut harness = page(unlocked());
    let state = harness.state_mut();
    let snapshot = std::sync::Arc::clone(&state.fixture.snapshot);
    state
        .fixture
        .changes
        .want::<RemovedUsers>(name("ada"), HomeFolder::Keep, &snapshot);
    state.fixture.revision += 1;
    harness.run();

    harness
        .get_by_role_and_label(Role::ListItem, "ada, Ada Lovelace")
        .focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    assert!(harness.state().page.dialog.is_none());
}

#[test]
fn return_on_a_row_opens_edit() {
    let mut harness = page(unlocked());
    harness
        .get_by_role_and_label(Role::ListItem, "ada, Ada Lovelace")
        .focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    assert!(harness.state().page.dialog.is_some());
}

#[test]
fn a_frame_with_the_same_inputs_builds_no_group_rows() {
    let mut harness = page(unlocked());
    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();
    let built = harness.state().page.groups.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.groups.builds(), built);

    harness.state_mut().page.shown.group_order = GroupOrder::Id;
    harness.run();
    assert_eq!(harness.state().page.groups.builds(), built + 1);

    let state = harness.state_mut();
    let snapshot = std::sync::Arc::clone(&state.fixture.snapshot);
    state
        .fixture
        .changes
        .want::<NewGroups>(group_name("audio"), Queued, &snapshot);
    state.fixture.revision += 1;
    harness.run();
    assert_eq!(harness.state().page.groups.builds(), built + 2);
}
