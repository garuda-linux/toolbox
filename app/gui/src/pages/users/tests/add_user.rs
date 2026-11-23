use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::users::run::RunKind;

use super::dialogs::type_into;
use super::{name, page, unlocked};

fn login(harness: &super::Shown) -> String {
    harness
        .get_by_role_and_label(Role::TextInput, "User name")
        .value()
        .unwrap_or_default()
}

#[test]
fn the_full_name_proposes_the_login_until_it_is_typed() {
    let mut harness = page(unlocked());
    harness.get_by_label("Add User…").click();
    harness.run();

    type_into(&mut harness, Role::TextInput, "Full name", "Grace Hopper");
    assert_eq!(login(&harness), "grace");

    type_into(&mut harness, Role::TextInput, "User name", "h");
    type_into(&mut harness, Role::TextInput, "Full name", " Jr");
    assert_eq!(login(&harness), "graceh");
}

/// The dialog closed when it sent the account. When the run fails it opens
/// again as it was sent, with both password fields empty.
#[test]
fn a_failed_new_account_opens_the_dialog_again_without_its_passwords() {
    let mut harness = page(unlocked());
    harness.get_by_label("Add User…").click();
    harness.run();
    type_into(&mut harness, Role::TextInput, "Full name", "Grace Hopper");
    type_into(&mut harness, Role::PasswordInput, "Password", "s3cret");
    type_into(
        &mut harness,
        Role::PasswordInput,
        "Repeat the password",
        "s3cret",
    );
    harness.get_by_label("Add User Now").click();
    harness.run();

    assert!(harness.query_by_label("Add User Now").is_none());

    let users = &mut harness.state_mut().fixture.services.users;
    users.started(RunKind::NewAccount, name("grace"));
    harness.run();
    harness.state_mut().fixture.services.users.plan_started();
    harness.run();
    harness
        .state_mut()
        .fixture
        .services
        .users
        .plan_finished(false);
    harness.run();

    assert!(harness.query_by_label("Add User Now").is_some());
    assert_eq!(login(&harness), "grace");
    assert!(harness.query_by_label("Enter a password.").is_some());
}

#[test]
fn a_new_account_that_was_added_leaves_the_dialog_closed() {
    let mut harness = page(unlocked());
    harness.get_by_label("Add User…").click();
    harness.run();
    type_into(&mut harness, Role::TextInput, "User name", "grace");
    type_into(&mut harness, Role::PasswordInput, "Password", "s3cret");
    type_into(
        &mut harness,
        Role::PasswordInput,
        "Repeat the password",
        "s3cret",
    );
    harness.get_by_label("Add User Now").click();
    harness.run();

    let users = &mut harness.state_mut().fixture.services.users;
    users.started(RunKind::NewAccount, name("grace"));
    users.plan_started();
    harness.run();
    harness
        .state_mut()
        .fixture
        .services
        .users
        .plan_finished(true);
    harness.run();

    assert!(harness.query_by_label("Add User Now").is_none());
}

/// The note is about the whole dialog, so it leads instead of reading as
/// the description of More options.
#[test]
fn the_note_that_the_user_is_added_at_once_leads_the_dialog() {
    let mut harness = page(unlocked());
    harness.get_by_label("Add User…").click();
    harness.run();

    let note = harness
        .get_by_label_contains("The user is created right away")
        .rect();

    let full_name = harness
        .get_by_role_and_label(Role::TextInput, "Full name")
        .rect();

    assert!(note.bottom() < full_name.top(), "{note:?} {full_name:?}");
}
