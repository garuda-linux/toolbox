use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_plan::MAX_PASSWORD_BYTES;

use super::dialogs::type_into;
use super::from_menu;

#[test]
fn a_pasted_password_past_the_limit_says_it_was_cut() {
    let mut harness = from_menu("ada", "Set password…");

    let long = "x".repeat(MAX_PASSWORD_BYTES + 40);
    type_into(&mut harness, Role::PasswordInput, "Password", &long);

    assert!(
        harness
            .query_by_label("The password is too long.")
            .is_some()
    );
}

/// The dialog opened before the plan started, so its button says why it
/// waits.
#[test]
fn set_password_now_says_why_it_waits_while_a_plan_runs() {
    let mut harness = from_menu("ada", "Set password…");
    type_into(&mut harness, Role::PasswordInput, "Password", "s3cret");
    type_into(
        &mut harness,
        Role::PasswordInput,
        "Repeat the password",
        "s3cret",
    );

    harness.state_mut().fixture.busy = true;
    harness.run();

    assert!(
        harness
            .query_by_label("Waits until the changes being applied are done.")
            .is_some()
    );
}
