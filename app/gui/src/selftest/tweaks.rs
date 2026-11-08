//! System Tweaks on the real system: the choices lead the catalog, and
//! the search reaches both. Which choices this machine reports is not
//! known here, so the dropdowns are left to the kittest tests.

use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    let [field, hblock] = Step::type_into("Search this page", "hblock");

    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num8),
        Step::wait("Login shell"),
        Step::Shot("tweaks"),
        field,
        hblock,
        Step::wait("Ad blocking"),
        Step::WaitGone("Login shell"),
        Step::Shot("tweaks-search"),
    ]
}
