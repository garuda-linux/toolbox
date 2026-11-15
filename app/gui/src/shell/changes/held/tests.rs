use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::unread;
use crate::command::{Command, CommandReceiver, PackagesCommand, UsersCommand, channel};

fn shown(reason: &'static str) -> egui_kittest::Harness<'static, CommandReceiver> {
    let (sender, receiver) = channel();
    let mut harness = toolbox_gui_kit::testing::harness(500.0, 200.0).build_ui_state(
        move |ui, _: &mut CommandReceiver| unread(ui, reason, &sender),
        receiver,
    );

    harness.run();
    harness
}

fn read_again(harness: &mut egui_kittest::Harness<'static, CommandReceiver>) -> Vec<Command> {
    harness
        .get_by_role_and_label(Role::Button, "Read again")
        .click();
    harness.run();
    harness.state().drain().map(|sent| sent.command).collect()
}

#[test]
fn locks_that_could_not_be_read_are_read_again_on_the_button() {
    let mut harness = shown("changes-unread-locks");

    assert!(
        harness
            .query_by_label_contains("could not be read")
            .is_some()
    );
    assert_eq!(
        read_again(&mut harness),
        [Command::from(UsersCommand::Refresh)]
    );
}

#[test]
fn an_aur_list_that_could_not_be_read_is_read_again_on_the_button() {
    let mut harness = shown("changes-unread-aur");

    assert_eq!(
        read_again(&mut harness),
        [Command::from(PackagesCommand::Refresh)]
    );
}

/// A reason that still waits on its own offers nothing to press.
#[test]
fn a_reason_that_waits_shows_no_button() {
    let harness = shown("changes-wait-locks");

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Read again")
            .is_none()
    );
}

/// The read the button started holds Apply with a wait reason. The banner
/// stays meanwhile, so the popover keeps its size.
#[test]
fn the_banner_stays_while_the_part_is_read_again() {
    let (sender, receiver) = channel();
    let reason = std::rc::Rc::new(std::cell::Cell::new("changes-unread-locks"));
    let shown_reason = std::rc::Rc::clone(&reason);
    let mut harness = toolbox_gui_kit::testing::harness(500.0, 200.0).build_ui_state(
        move |ui, _: &mut CommandReceiver| unread(ui, shown_reason.get(), &sender),
        receiver,
    );

    harness.run();
    let before = harness
        .get_by_role_and_label(Role::Button, "Read again")
        .rect();

    read_again(&mut harness);

    reason.set("changes-wait-locks");
    harness.run();

    let during = harness
        .get_by_role_and_label(Role::Button, "Read again")
        .rect();

    assert_eq!(during, before);

    assert!(
        harness
            .query_by_label_contains("could not be read")
            .is_some()
    );
}

/// A wait nobody asked for is a first read, which the page tells.
#[test]
fn a_first_wait_shows_no_banner() {
    let harness = shown("changes-wait-locks");

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Read again")
            .is_none()
    );
}
