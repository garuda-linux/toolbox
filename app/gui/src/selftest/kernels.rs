//! The Kernels page: open it on the real system, wait for dkms and read
//! the kernels again. Which kernels a machine has, or whether it has
//! pacman at all, is not known here, so the queuing is left to the
//! kittest tests.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num9),
        Step::WaitContaining("A kernel you install is used after the next restart"),
        Step::WaitContaining("Running kernel: "),
        Step::WaitGone("Asking dkms for its modules…"),
        Step::Shot("kernels"),
        Step::click("Read again"),
        Step::WaitGone("Reading the system…"),
        Step::WaitGone("Asking dkms for its modules…"),
        Step::Shot("kernels-refreshed"),
    ]
}
