use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::users::account::AccountEdit;
use toolbox_plan::FullName;

use super::dialogs::type_into;
use super::{name, page, sent, unlocked};
use crate::command::{Command, UserEdit};

fn edit_dialog(row: &str, user: &str) -> super::Shown {
    let mut harness = page(unlocked());
    harness.get_by_label(row).click();
    harness.run();
    harness.get_by_label(&format!("Edit {user}…")).click();
    harness.run();
    harness
}

#[test]
fn the_edit_dialog_queues_only_what_changed() {
    let mut harness = edit_dialog("ada, Ada Lovelace", "ada");

    type_into(&mut harness, Role::TextInput, "Full name", " King");
    harness.get_by_label("Queue changes").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(UserEdit::Set {
            user: name("ada"),
            edit: AccountEdit {
                full_name: Some(FullName::new("Ada Lovelace King").unwrap()),
                ..AccountEdit::default()
            },
        })]
    );
}

/// The guard runs before Queue changes: the own account's home folder
/// says why it stays where its field would be.
#[test]
fn the_own_accounts_home_folder_says_why_it_stays() {
    let harness = edit_dialog("nico, Nico", "nico");

    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Home folder")
            .is_none()
    );
    assert!(
        harness
            .query_by_label_contains("account you are logged in with cannot change")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Full name")
            .is_some()
    );
}

#[test]
fn every_field_of_the_edit_dialog_keeps_one_gap_to_the_next() {
    let harness = edit_dialog("ada, Ada Lovelace", "ada");

    let full_name = harness
        .get_by_role_and_label(Role::TextInput, "Full name")
        .rect();

    let shell = harness
        .get_by_role_and_label(Role::Label, "Login shell")
        .rect();

    let shell_hint = harness.get_by_label_contains("Only shells listed").rect();
    let home = harness
        .get_by_role_and_label(Role::Label, "Home folder")
        .rect();

    let after_field = shell.top() - full_name.bottom();
    let after_hint = home.top() - shell_hint.bottom();
    assert!(
        (after_field - after_hint).abs() < 1.0,
        "{after_field} px after a field, {after_hint} px after a hint"
    );
}
