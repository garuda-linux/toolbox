//! The scroll handles a frame paints, for the tests of the bars.

use egui_kittest::Harness;

use super::rects;
use crate::theme::{MIN_LENGTH, REST_WIDTH};

/// The scroll handles the last frame painted: a filled rectangle as wide as
/// the handle at rest and at least its minimum length long.
pub fn scroll_handles<State>(harness: &Harness<'_, State>) -> Vec<egui::Rect> {
    rects(harness)
        .filter(|painted| {
            painted.fill.a() > 0
                && (painted.rect.width() - REST_WIDTH).abs() < 0.01
                && painted.rect.height() >= MIN_LENGTH - 0.01
        })
        .map(|painted| painted.rect)
        .collect()
}
