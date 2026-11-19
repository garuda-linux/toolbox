//! Home on the real system: the greeting with its stat tiles, the status once
//! checked, the dry run's sample news, the merge dialog with the self-test's
//! own copies, then a fresh check through the heading's button. A page
//! opened before comes back as a recent chip.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("About"),
        Step::wait("Copy system details"),
        Step::shortcut(Key::Num1),
        Step::WaitGone("Checking the system…"),
        Step::WaitContaining("Disk: "),
        Step::WaitContaining("Updates: "),
        Step::wait("News"),
        Step::wait("Open Toolbox sample: a new kernel series"),
        Step::wait("Manual step"),
        Step::Shot("home"),
        Step::button("Review…"),
        Step::wait("/etc/locale.gen.pacsave beside /etc/locale.gen"),
        Step::wait("The file in use is gone. The copy can take its place."),
        Step::button("Restore saved file"),
        Step::WaitContaining(", queued"),
        Step::Shot("home-merges-gone"),
        Step::button("Take back"),
        Step::tab("/etc/pacman.conf"),
        Step::wait("/etc/pacman.conf.pacnew beside /etc/pacman.conf"),
        Step::disabled_button("Queue the merge"),
        Step::radio("Take new"),
        Step::button("Queue the merge"),
        Step::WaitContaining(", queued"),
        Step::Shot("home-merges"),
        Step::button("Take back"),
        Step::press(Key::Escape),
        Step::WaitGone("Files"),
        Step::click("Read again"),
        Step::WaitGone("Checking the system…"),
        Step::wait("Next steps"),
        Step::wait("Recently used"),
        Step::Shot("home-checked"),
        Step::button("About"),
        Step::wait("Copy system details"),
    ]
}
