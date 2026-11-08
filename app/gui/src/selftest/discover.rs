use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    let [field, firefox] = Step::type_into("Search apps", "firefox");
    let [field_again, nothing] = Step::type_into("Search apps", "-nothing");

    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num5),
        Step::wait("Sort apps"),
        Step::Shot("discover"),
        field,
        firefox,
        Step::click_checkbox("Firefox"),
        Step::wait("Turns on with Apply"),
        Step::Shot("discover-queued"),
        field_again,
        nothing,
        Step::WaitContaining("No app matches"),
        Step::Shot("discover-nothing"),
    ]
}
