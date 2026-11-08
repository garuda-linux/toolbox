//! Home on the real system: the greeting with its facts, the status once
//! checked, the dry run's sample news, then a fresh check through the
//! heading's button. A page opened before comes back as a recent chip.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("About"),
        Step::wait("Version"),
        Step::shortcut(Key::Num1),
        Step::WaitGone("Checking the system…"),
        Step::WaitContaining("free on /"),
        Step::WaitContaining(" · up "),
        Step::wait("News"),
        Step::wait("Open Toolbox sample: a new kernel series"),
        Step::wait("Manual step"),
        Step::Shot("home"),
        Step::click("Read again"),
        Step::WaitGone("Checking the system…"),
        Step::wait("Next steps"),
        Step::wait("Recently used"),
        Step::Shot("home-checked"),
        Step::button("About"),
        Step::wait("Version"),
    ]
}
