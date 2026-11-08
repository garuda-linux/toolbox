//! The Maintenance page: an operation queued and taken back, another applied
//! through the dry-run helper, then the reset and Repairs tabs. A run starts
//! no program and changes no file of the home. Earlier suites may leave
//! changes queued, so the waits look at this page's own buttons.

use egui::Key;

use super::step::Step;

const ORPHANS: &str = "More actions for Remove orphans";
const CACHE: &str = "More actions for Clean package cache";

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Maintenance"),
        Step::wait("Package care"),
        Step::Shot("maintenance"),
        // Queue sits in each row's ⋯ menu, behind Run now. The menu must be
        // gone before it opens again, or the click only closes it.
        Step::click(ORPHANS),
        Step::click("Queue removal"),
        Step::WaitGone("Queue removal"),
        Step::click(ORPHANS),
        Step::wait("Remove from queue"),
        Step::Shot("maintenance-menu"),
        Step::click("Remove from queue"),
        Step::WaitGone("Remove from queue"),
        Step::click(ORPHANS),
        Step::wait("Queue removal"),
        Step::press(Key::Escape),
        Step::WaitGone("Queue removal"),
        Step::click(CACHE),
        Step::click("Queue cleanup"),
        Step::WaitGone("Queue cleanup"),
        Step::click(CACHE),
        Step::wait("Remove from queue"),
        Step::press(Key::Escape),
        Step::WaitGone("Remove from queue"),
        Step::click("Apply"),
        Step::wait("Test mode: nothing was changed"),
        // After the run nothing is queued any more. The Escape that closes
        // the menu leaves the Console open.
        Step::click(CACHE),
        Step::wait("Queue cleanup"),
        Step::press(Key::Escape),
        Step::WaitGone("Queue cleanup"),
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
