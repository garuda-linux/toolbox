//! The Users and groups page on the real system: the accounts, the add
//! dialog refusing a bad name, and the groups. A dry run's helper cannot
//! read `/etc/shadow`, so the page says the lock state is not known.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    let [field, bad_name] = Step::type_into("User name", "Bad Name");

    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Users and Groups"),
        Step::WaitEither("Add user…", "User accounts are not available"),
        Step::WaitContaining("Showing"),
        Step::WaitGone("Asking which accounts are locked…"),
        Step::Shot("users"),
        Step::button("Add user…"),
        field,
        bad_name,
        Step::WaitContaining("Names start with a lowercase letter"),
        Step::disabled_button("Add user now"),
        Step::Shot("users-add"),
        Step::press(Key::Escape),
        Step::WaitGone("Add a user"),
        Step::radio("Groups"),
        Step::WaitContaining("Group ID"),
        Step::Shot("users-groups"),
    ]
}
