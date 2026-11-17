use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Queued;
use toolbox_core::domains::{Accounts, NewGroups, RemovedUsers};
use toolbox_core::features::users::listing::{GroupOrder, UserOrder};
use toolbox_core::features::users::run::PageRun;
use toolbox_core::testing::Queue as _;
use toolbox_plan::HomeFolder;
use toolbox_services::locks::Locks;

use super::{group_name, name, page, page_on, sent, system, unlocked, user};
use crate::command::{Command, UsersCommand};
use crate::users::Users;

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
    harness.state_mut().page.shown.user_order = UserOrder::Name;
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
    harness.state_mut().page.shown.user_order = UserOrder::Name;
    harness.run();
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

    harness.state_mut().page.shown.group_order = GroupOrder::Name;
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

#[test]
fn the_keyboard_hint_shows_while_a_row_has_the_focus() {
    let mut harness = page(unlocked());
    harness.state_mut().page.shown.user_order = UserOrder::Name;
    harness.run();
    let hint = "Arrow keys pick an account. Return edits it, the Menu key opens its other actions.";
    assert!(harness.query_by_label(hint).is_none());

    harness
        .get_by_role_and_label(Role::ListItem, "ada, Ada Lovelace")
        .focus();
    harness.run();

    assert!(harness.query_by_label(hint).is_some());
}

/// The lock state belongs to accounts, so the Groups tab does not repeat
/// it, and the banner reads it again itself.
#[test]
fn an_unknown_lock_state_offers_read_again_on_the_users_tab_only() {
    let mut harness = page(Users::showing(Locks::Unknown, PageRun::None));
    let banner = "Which accounts are locked is not known";

    // The header's button and the banner's.
    let buttons = || harness.query_all_by_role_and_label(Role::Button, "Read again");
    assert_eq!(buttons().count(), 2);
    buttons().last().unwrap().click();
    harness.run();
    let read = sent(&harness);
    assert!(
        read.contains(&Command::from(UsersCommand::Refresh)),
        "{read:?}"
    );
    assert!(harness.query_by_label_contains(banner).is_some());

    harness
        .get_by_role_and_label(Role::RadioButton, "Groups")
        .click();
    harness.run();

    assert!(harness.query_by_label_contains(banner).is_none());
}

#[test]
fn the_root_note_stays_inside_its_row() {
    let mut snapshot = system();
    let mut accounts = snapshot.part::<Accounts>().unwrap().clone();
    accounts.users.push(user("root", 0, "root"));
    snapshot.insert(accounts);
    let mut harness = page_on(snapshot, unlocked());
    harness
        .get_by_role_and_label(Role::Button, "Regular users only")
        .click();
    harness.run();

    let row = harness
        .get_by_role_and_label(Role::ListItem, "root, root")
        .rect();

    let note = harness
        .get_by_label("Root accounts are not changed here.")
        .rect();

    assert!(row.contains_rect(note), "{note:?} outside {row:?}");
}
