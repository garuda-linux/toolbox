//! The About page: the hero, the facts and the links show, the system
//! details are copied, and the exorcist answers in a dialog that Escape
//! closes. Ctrl+Plus zooms the window, Ctrl+Minus stops at the smallest
//! zoom and Ctrl+0 resets it. No link is clicked, a run must not open
//! the browser.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("About"),
        Step::wait("Copy System Details"),
        Step::wait("Garuda Linux wiki"),
        Step::Shot("about"),
        Step::button("Copy System Details"),
        Step::wait("Copied the system details to the clipboard."),
        Step::button("Call the Exorcist"),
        Step::WaitContaining("special voodoo"),
        Step::wait("Issue tracker"),
        Step::Shot("about-exorcist"),
        Step::press(Key::Escape),
        Step::WaitGone("Issue tracker"),
        Step::shortcut(Key::Plus),
        Step::wait("Copy System Details"),
        Step::Shot("about-zoomed"),
        Step::shortcut(Key::Minus),
        Step::shortcut(Key::Minus),
        Step::shortcut(Key::Minus),
        Step::shortcut(Key::Minus),
        Step::wait("Copy System Details"),
        Step::Shot("about-zoom-smallest"),
        Step::shortcut(Key::Num0),
        Step::wait("Copy System Details"),
    ]
}
