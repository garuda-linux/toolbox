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
        Step::click("Firefox"),
        Step::wait("Back to App Catalog"),
        Step::Shot("discover-details"),
        Step::click("Back to App Catalog"),
        Step::wait("Install Firefox"),
        Step::press(Key::Enter),
        Step::wait("Back to App Catalog"),
        Step::press(Key::Escape),
        Step::wait("Install Firefox"),
        Step::button("Install Firefox"),
        Step::wait("Installs with Apply"),
        Step::Shot("discover-queued"),
        field_again,
        nothing,
        Step::WaitContaining("Nothing matches"),
        Step::Shot("discover-nothing"),
    ]
}
