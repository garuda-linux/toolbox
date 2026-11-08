//! The User guide: it opens on the introduction, F1 on Kernels opens the
//! guide's Kernels page, and a search that finds nothing says so. No
//! link to the web is clicked, a run must not open the browser.

use egui::Key;
use egui::accesskit::Role;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("User guide"),
        Step::wait("Search the guide"),
        Step::Shot("guide"),
        Step::tab("Kernels"),
        Step::WaitContaining("A kernel you install is used after the next restart"),
        Step::press(Key::F1),
        Step::wait("Open Kernels"),
        Step::Shot("guide-f1"),
        Step::Click {
            role: Some(Role::TextInput),
            label: "Search the guide",
        },
        Step::Text("qwertzuiop"),
        Step::WaitContaining("Nothing in the guide matches"),
        Step::Shot("guide-no-hits"),
    ]
}
