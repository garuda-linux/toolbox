//! A thin bar of how much of something is used, such as the disk. It only
//! adorns a figure the text says already, so screen readers skip it.

use egui::{Color32, Rect, Ui, vec2};

use super::Tone;
use crate::tokens::{self, Tokens};

/// Without a share the line keeps its room and shows nothing, so a row
/// of tiles with and without one stays even.
pub fn meter(ui: &mut Ui, share: Option<f32>, tone: Tone) -> Rect {
    let tokens = tokens::current(ui.ctx());
    let height = tokens.sizes.meter_height;
    let (_, rect) = ui.allocate_space(vec2(ui.available_width(), height));
    let Some(share) = share else {
        return rect;
    };

    if !ui.is_rect_visible(rect) {
        return rect;
    }

    let radius = height / 2.0;
    let painter = ui.painter();
    painter.rect_filled(rect, radius, tokens.field);

    // A sliver keeps the round ends, so a nearly empty disk still shows.
    let share = share.clamp(0.0, 1.0);
    if share > 0.0 {
        let width = (rect.width() * share).max(height).min(rect.width());
        let filled = Rect::from_min_size(rect.min, vec2(width, height));
        painter.rect_filled(filled, radius, fill(tone, &tokens));
    }

    rect
}

/// The accent fitted to 3:1 on the track while fine, as the focus ring
/// is: the bar is the only mark of the share.
const fn fill(tone: Tone, tokens: &Tokens) -> Color32 {
    match tone {
        Tone::Accent => tokens.accent_ring,
        Tone::Neutral | Tone::Warning | Tone::Success | Tone::Error => tone.status_colour(tokens),
    }
}
