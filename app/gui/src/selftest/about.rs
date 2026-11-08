//! The About page: the facts and the links show, and the exorcist answers
//! in a dialog that Escape closes. No link is clicked, a run must not open
//! the browser.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("About"),
        Step::wait("Version"),
        Step::wait("Garuda Linux wiki"),
        Step::Shot("about"),
        Step::button("Call the exorcist"),
        Step::WaitContaining("special voodoo"),
        Step::wait("Issue tracker"),
        Step::Shot("about-exorcist"),
        Step::press(Key::Escape),
        Step::WaitGone("Issue tracker"),
    ]
}
