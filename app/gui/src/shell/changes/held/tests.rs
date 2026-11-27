use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::unread;
use crate::command::{Command, CommandReceiver, PackagesCommand, UsersCommand, channel};
use crate::shell::Hold;

/// A hold whose sentence is not needed by the reasons that look it up.
fn hold(id: &'static str) -> Hold {
    Hold {
        id,
        text: String::new(),
    }
}

fn shown(reason: &'static str) -> egui_kittest::Harness<'static, CommandReceiver> {
    let (sender, receiver) = channel();
    let mut harness = toolbox_gui_kit::testing::harness(500.0, 200.0).build_ui_state(
        move |ui, _: &mut CommandReceiver| unread(ui, &hold(reason), &sender),
        receiver,
    );

    harness.run();
    harness
}

fn read_again(harness: &mut egui_kittest::Harness<'static, CommandReceiver>) -> Vec<Command> {
    harness
        .get_by_role_and_label(Role::Button, "Read Again")
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

/// The banner of a build without its order names the package, and Read again
/// reads the orders of the queue once more.
#[test]
fn a_build_without_its_order_names_it_and_read_again_reads_the_orders() {
    let (sender, receiver) = channel();
    let named = Hold {
        id: "changes-wait-orders",
        text: "Apply waits for a build order. no-such-package has no build order: the AUR does not have it.".to_owned(),
    };

    let mut harness = toolbox_gui_kit::testing::harness(500.0, 200.0).build_ui_state(
        move |ui, _: &mut CommandReceiver| unread(ui, &named, &sender),
        receiver,
    );

    harness.run();

    assert!(
        harness
            .query_by_label_contains("no-such-package has no build order")
            .is_some()
    );
    assert_eq!(
        read_again(&mut harness),
        [Command::from(PackagesCommand::ReadOrders)]
    );
}

/// A reason that still waits on its own offers nothing to press.
#[test]
fn a_reason_that_waits_shows_no_button() {
    let harness = shown("changes-wait-locks");

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Read Again")
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
        move |ui, _: &mut CommandReceiver| unread(ui, &hold(shown_reason.get()), &sender),
        receiver,
    );

    harness.run();
    let before = harness
        .get_by_role_and_label(Role::Button, "Read Again")
        .rect();

    read_again(&mut harness);

    reason.set("changes-wait-locks");
    harness.run();

    let during = harness
        .get_by_role_and_label(Role::Button, "Read Again")
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
            .query_by_role_and_label(Role::Button, "Read Again")
            .is_none()
    );
}
