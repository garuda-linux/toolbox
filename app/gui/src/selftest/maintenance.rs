//! The Maintenance page: an operation queued and taken back, another applied
//! through the dry-run helper, then the reset and Repairs tabs. A run starts
//! no program and changes no file of the home. Earlier suites may leave
//! changes queued, so the waits look at this page's own buttons.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Maintenance"),
        Step::wait("Package care"),
        Step::Shot("maintenance"),
        // Each row's Queue sits beside its Run now and turns into Remove
        // from queue while the task waits.
        Step::button("Queue removal"),
        Step::wait("Remove from queue"),
        Step::Shot("maintenance-queued"),
        Step::button("Remove from queue"),
        Step::WaitGone("Remove from queue"),
        Step::button("Queue cleanup"),
        Step::wait("Remove from queue"),
        Step::click("Apply"),
        Step::wait("Test mode: nothing was changed"),
        // A test run skips every step, so the cleanup stays queued.
        Step::wait("Remove from queue"),
        Step::Shot("maintenance-applied"),
        // A test run prints nothing, so the Console's menu offers only to
        // clear it. Sharing is covered by the integration tests.
        Step::click("Console output actions"),
        Step::wait("Clear output"),
        Step::Shot("console-menu"),
        Step::click("Clear output"),
        Step::WaitGone("Test mode: nothing was changed"),
        Step::wait("No changes are being applied"),
        Step::shortcut(Key::J),
        Step::WaitGone("No changes are being applied"),
        Step::radio("Reset settings"),
        Step::wait("Application settings"),
        Step::Shot("maintenance-reset"),
        // The palette opens the tab that holds a repair.
        Step::shortcut(Key::K),
        Step::Text("reset audio"),
        Step::WaitContaining("Maintenance · Repairs"),
        Step::Shot("maintenance-palette"),
        Step::press(Key::Enter),
        Step::wait("Repairs"),
        Step::Shot("maintenance-repairs"),
    ]
}
