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
        // The self-test window is wide, so the details open beside the list
        // and no Back button shows.
        Step::click("Firefox"),
        Step::wait("Open Website"),
        Step::Shot("discover-details"),
        Step::press(Key::Escape),
        Step::WaitGone("Open Website"),
        Step::press(Key::Enter),
        Step::wait("Open Website"),
        Step::button("Install Firefox"),
        Step::wait("Installs with Apply"),
        Step::Shot("discover-queued"),
        field_again,
        nothing,
        Step::WaitContaining("Nothing matches"),
        Step::Shot("discover-nothing"),
    ]
}
