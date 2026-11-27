//! The Settings page steps. The changes go to the run's own settings file,
//! never the user's.

use egui::Key;
use egui::accesskit::Role;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Settings"),
        Step::wait("Appearance"),
        Step::Shot("settings"),
        Step::click("Light"),
        Step::Shot("settings-light"),
        Step::click("As the desktop"),
        Step::Click {
            role: Some(Role::TextInput),
            label: "Minutes between checks",
        },
        Step::shortcut(Key::A),
        Step::Text("2"),
        Step::press(Key::Enter),
        Step::WaitContaining("so this is 5 minutes"),
        Step::Shot("settings-held"),
        Step::click_checkbox("Check for updates automatically"),
        Step::WaitGone("Time between checks"),
        Step::Shot("settings-checks-off"),
        Step::click_checkbox("Check for updates automatically"),
        Step::wait("Time between checks"),
        Step::combo("Dark color scheme"),
        option("Nord"),
        Step::Shot("settings-scheme-dark"),
        Step::combo("Dark color scheme"),
        option("System"),
        Step::Shot("settings-scheme-system"),
        Step::click("Light"),
        Step::combo("Light color scheme"),
        option("Catppuccin Latte"),
        Step::Shot("settings-scheme-light"),
        Step::combo("Light color scheme"),
        option("System"),
        Step::Disabled {
            role: Some(Role::CheckBox),
            label: "Blur the desktop behind the window",
        },
        Step::WaitContaining("This desktop does not offer blur behind windows"),
        Step::Shot("settings-blur-unavailable"),
        Step::click("As the desktop"),
    ]
}

/// An entry of the open dropdown.
const fn option(label: &'static str) -> Step {
    Step::Click {
        role: Some(Role::ListBoxOption),
        label,
    }
}
