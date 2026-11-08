//! The one focus ring every painted widget shares, so keyboard users see
//! where they are on every control alike.

use egui::{CornerRadius, Rect, Response, Stroke, StrokeKind, Ui};

use crate::tokens;

const GAP: u8 = 2;

pub fn focus_ring(ui: &Ui, response: &Response, radius: u8) {
    if response.has_focus() {
        ring_around(ui, response.rect, radius);
    }
}

/// The ring around `rect`, for a control whose focus sits on a part of
/// it, such as a search field's text.
pub(super) fn ring_around(ui: &Ui, rect: Rect, radius: u8) {
    let accent = tokens::current(ui.ctx()).accent_ring;
    ui.painter().rect_stroke(
        rect.expand(f32::from(GAP)),
        CornerRadius::same(radius.saturating_add(GAP)),
        Stroke::new(2.0, accent),
        StrokeKind::Outside,
    );
}
