use egui::accesskit::Role;
use egui::text_edit::TextEditState;
use egui::util::IdTypeMap;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::domains::Accounts;
use toolbox_core::features::users::input::{self, NewAccount};
use toolbox_plan::{FullName, Password};

use super::{group, group_name, name, page, page_on, sent, system, unlocked};
use crate::command::{Command, UserEdit, UsersCommand};
use crate::pages::Refusals;

#[test]
fn a_new_password_runs_once_both_fields_agree() {
    let mut harness = page(unlocked(), Refusals::default());

    harness.get_by_label("ada, Ada Lovelace").click();
    harness.run();
    harness.get_by_label("More actions for ada").click();
    harness.run();
    harness.get_by_label("Set password…").click();
    harness.run();
    type_into(&mut harness, Role::PasswordInput, "Password", "s3cret");
    type_into(
        &mut harness,
        Role::PasswordInput,
        "Repeat the password",
        "s3cre",
    );

    assert!(
        harness
            .query_by_label("The two passwords differ.")
            .is_some()
    );

    type_into(
        &mut harness,
        Role::PasswordInput,
        "Repeat the password",
        "t",
    );
    harness.get_by_label("Set password now").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(UsersCommand::SetPassword {
            user: name("ada"),
            password: Password::new("s3cret".to_owned()).unwrap(),
        })]
    );
}

#[test]
fn a_closed_password_dialog_leaves_no_field_state_behind() {
    let mut harness = page(unlocked(), Refusals::default());
    let states = |shown: &super::Shown| shown.ctx.data(IdTypeMap::count::<TextEditState>);

    harness.get_by_label("ada, Ada Lovelace").click();
    harness.run();
    harness.get_by_label("More actions for ada").click();
    harness.run();
    let before = states(&harness);
    harness.get_by_label("Set password…").click();
    harness.run();
    type_into(&mut harness, Role::PasswordInput, "Password", "s3cret");
    type_into(
        &mut harness,
        Role::PasswordInput,
        "Repeat the password",
        "s3",
    );

    assert!(states(&harness) > before, "the fields keep a state");

    harness
        .get_by_role_and_label(Role::Button, "Cancel")
        .click();
    harness.run();

    assert_eq!(states(&harness), before);
}

#[test]
fn the_edit_dialog_queues_only_what_changed() {
    let mut harness = page(unlocked(), Refusals::default());

    harness.get_by_label("ada, Ada Lovelace").click();
    harness.run();
    harness.get_by_label("Edit ada…").click();
    harness.run();
    type_into(&mut harness, Role::TextInput, "Full name", " King");
    harness.get_by_label("Queue changes").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(UserEdit::SetFullName {
            user: name("ada"),
            name: FullName::new("Ada Lovelace King").unwrap(),
        })]
    );
}

#[test]
fn a_taken_name_is_refused_while_it_is_typed() {
    let mut harness = page(unlocked(), Refusals::default());

    harness.get_by_label("Add user…").click();
    harness.run();
    type_into(&mut harness, Role::TextInput, "User name", "ada");

    assert!(
        harness
            .query_by_label("A user named ada exists already.")
            .is_some()
    );
}

#[test]
fn a_new_user_is_added_with_the_groups_ticked() {
    let mut harness = page(unlocked(), Refusals::default());

    harness.get_by_label("Add user…").click();
    harness.run();
    type_into(&mut harness, Role::TextInput, "User name", "grace");
    type_into(&mut harness, Role::PasswordInput, "Password", "s3cret");
    type_into(
        &mut harness,
        Role::PasswordInput,
        "Repeat the password",
        "s3cret",
    );
    harness.get_by_label("More options").click();
    harness.run();

    // As Tab reaches it: the box gets the focus and the dialog scrolls the
    // list into sight.
    harness
        .get_by_role_and_label(Role::CheckBox, "video")
        .focus();
    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "video")
        .click();
    harness.run();
    harness.get_by_label("Add user now").click();
    harness.run();

    let form = NewAccount {
        name: "grace",
        full_name: "",
        shell: "",
        groups: Some([group_name("video")].into()),
        home: "",
        no_home: false,
        uid: "",
        system: false,
    };

    let user = input::new_account(&form, &system()).unwrap();

    assert_eq!(
        sent(&harness),
        [Command::from(UsersCommand::AddUser {
            user: Box::new(user),
            password: Password::new("s3cret".to_owned()).unwrap(),
        })]
    );
}

/// Clicks the field `label` names, as typing needs the focus, and types
/// `text` at its end.
fn type_into(harness: &mut super::Shown, role: Role, label: &str, text: &str) {
    harness.get_by_role_and_label(role, label).click();
    harness.run();
    harness.get_by_role_and_label(role, label).type_text(text);
    harness.run();
}

#[test]
fn the_add_dialog_asks_the_required_fields_first_and_says_what_holds_it() {
    let mut harness = page(unlocked(), Refusals::default());
    harness.get_by_label("Add user…").click();
    harness.run();

    let top = |shown: &super::Shown, role, label: &str| {
        shown.get_by_role_and_label(role, label).rect().top()
    };

    harness.get_by_label("More options").click();
    harness.run();
    let name = top(&harness, Role::TextInput, "User name");
    let password = top(&harness, Role::PasswordInput, "Password");
    let admin = top(&harness, Role::CheckBox, "Administrator");
    let full_name = top(&harness, Role::TextInput, "Full name");
    assert!(
        name < password && password < admin && admin < full_name,
        "{name} {password} {admin} {full_name}"
    );

    assert!(harness.query_by_label("Enter a user name.").is_some());
    type_into(&mut harness, Role::TextInput, "User name", "grace");
    assert!(harness.query_by_label("Enter a password.").is_some());
    type_into(&mut harness, Role::PasswordInput, "Password", "s3cret");
    assert!(
        harness
            .query_by_label("The passwords do not match.")
            .is_some()
    );
}

#[test]
fn the_required_fields_fit_and_the_optional_ones_wait_folded() {
    let mut harness = page(unlocked(), Refusals::default());
    harness.set_size(egui::vec2(1024.0, 720.0));
    harness.run();
    harness.get_by_label("Add user…").click();
    harness.run();

    let confirm = harness.get_by_label("Add user now").rect();
    assert!(confirm.bottom() <= 720.0, "{confirm:?}");

    for (role, label) in [
        (Role::TextInput, "User name"),
        (Role::PasswordInput, "Password"),
        (Role::PasswordInput, "Repeat the password"),
        (Role::CheckBox, "Administrator"),
    ] {
        let field = harness.get_by_role_and_label(role, label).rect();
        assert!(
            field.bottom() < confirm.top(),
            "{label} is below the fold: {field:?}"
        );
    }

    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Full name")
            .is_none()
    );
    harness.get_by_label("More options").click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Full name")
            .is_some()
    );
}

#[test]
fn every_field_of_the_edit_dialog_keeps_one_gap_to_the_next() {
    let mut harness = page(unlocked(), Refusals::default());
    harness.get_by_label("ada, Ada Lovelace").click();
    harness.run();
    harness.get_by_label("Edit ada…").click();
    harness.run();

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

#[test]
fn more_options_folds_open_behind_a_chevron_on_the_label_column() {
    let mut harness = page(unlocked(), Refusals::default());
    harness.get_by_label("Add user…").click();
    harness.run();

    let more = harness
        .get_by_role_and_label(Role::Button, "More options")
        .rect();

    let column = harness
        .get_by_role_and_label(Role::TextInput, "User name")
        .rect()
        .left();

    let chevron = toolbox_gui_kit::testing::textured_rects(&harness)
        .into_iter()
        .any(|icon| more.contains_rect(icon) && icon.left() - more.left() < 1.0);

    assert!((more.left() - column).abs() < 1.0, "{more:?} {column}");
    assert!(chevron, "no chevron icon at the start of {more:?}");
    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Full name")
            .is_none()
    );

    harness.get_by_label("More options").click();
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Full name")
            .is_some()
    );
}

#[test]
fn many_groups_scroll_in_a_list_of_five_and_a_half_rows() {
    let mut snapshot = system();
    let mut accounts = snapshot
        .part::<Accounts>()
        .cloned()
        .expect("the fixture has accounts");

    for number in 0..20_u32 {
        accounts
            .groups
            .push(group(&format!("group{number:02}"), 2000 + number, &[]));
    }

    snapshot.insert(accounts);
    let mut harness = page_on(snapshot, unlocked(), Refusals::default());
    harness.get_by_label("Add user…").click();
    harness.run();
    harness.get_by_label("More options").click();
    harness.run();

    let first = harness
        .get_by_role_and_label(Role::CheckBox, "group00")
        .rect();

    let below = harness.get_by_label("Add user now").rect();
    let row = toolbox_gui_kit::widgets::CHECK_ROW_HEIGHT;

    assert!(
        below.top() - first.top() < row * 9.0,
        "the list runs {} down to the button",
        below.top() - first.top()
    );
}
