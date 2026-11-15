//! Whether the user looks at the window, which eframe's hidden frames
//! leave stale in `InputState::focused`.

use egui::{InputState, ViewportId};

use super::super::services::looked_at;

fn input(focused: bool, minimized: Option<bool>) -> InputState {
    let mut input = InputState::default();
    input.raw.focused = focused;
    if let Some(viewport) = input.raw.viewports.get_mut(&ViewportId::ROOT) {
        viewport.minimized = minimized;
    }

    input
}

#[test]
fn a_focused_shown_window_is_looked_at() {
    assert!(looked_at(&input(true, Some(false))));
}

#[test]
fn the_raw_focus_counts_over_the_last_shown_frame() {
    let mut hidden = input(false, None);
    hidden.focused = true;

    assert!(!looked_at(&hidden));
}

#[test]
fn a_minimized_window_is_not_looked_at_while_it_keeps_the_focus() {
    assert!(!looked_at(&input(true, Some(true))));
}
