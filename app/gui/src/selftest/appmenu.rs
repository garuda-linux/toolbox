//! The global menu over a private session bus: Review and apply… refuses
//! while no change waits, a page opens from Go, and View hides the
//! sidebar's page names. Weston shows no global menu, so the menu is only
//! served there, and the steps click it as Plasma would.

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::menu_refuses("Review and apply…"),
        Step::menu_click("Kernels"),
        Step::WaitContaining("A kernel you install is used after the next restart"),
        Step::Shot("appmenu-kernels"),
        Step::menu_click("Expanded sidebar"),
        Step::wait("Expand sidebar"),
        Step::Shot("appmenu-sidebar-collapsed"),
        Step::menu_click("Expanded sidebar"),
        Step::wait("Collapse sidebar"),
    ]
}
