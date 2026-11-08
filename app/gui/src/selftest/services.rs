//! Services on the real system: the system units, then the user's. Which
//! units a machine has is not known here, so the switches and the menu
//! are left to the kittest tests.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num7),
        Step::wait("Hide inactive"),
        Step::WaitGone("Reading the services…"),
        Step::Shot("services"),
        Step::click("User"),
        Step::WaitGone("Reading the services…"),
        Step::Shot("services-user"),
    ]
}
