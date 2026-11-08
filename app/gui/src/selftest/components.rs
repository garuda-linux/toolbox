use egui::Key;

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    let [search_field, no_match] = Step::type_into("Search this page", "zz-nothing-matches");

    vec![
        Step::WaitGone("Reading the system…"),
        Step::shortcut(Key::Num3),
        Step::wait("Audio"),
        Step::Shot("components"),
        search_field,
        no_match,
        Step::wait("Nothing matches the search"),
        Step::Shot("components-no-match"),
        Step::click("Clear search"),
        Step::wait("Audio"),
        Step::click_checkbox("PipeWire"),
        Step::WaitContaining("change queued"),
        Step::click("Review"),
        Step::WaitContaining("pipewire-support"),
        Step::Shot("components-queued"),
        Step::click("Apply the change"),
        Step::wait("Test mode: nothing was changed"),
        Step::Shot("components-applied"),
        // The Console sheet covers the next suite's page.
        Step::click("Hide the Console"),
        Step::WaitGone("Test mode: nothing was changed"),
    ]
}
