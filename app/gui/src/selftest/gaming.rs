//! The Gaming page: open it, find a launcher with the search and queue
//! it.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    // Steam is in multilib, which plain Arch leaves off.
    let [search_field, lutris] = Step::type_into("Search this page", "lutris");

    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num4),
        Step::wait("Game launchers"),
        Step::Shot("gaming"),
        search_field,
        lutris,
        Step::click_checkbox("Lutris"),
        Step::wait("Turns on with Apply"),
        Step::Shot("gaming-queued"),
    ]
}
