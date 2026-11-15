//! A row painted by hand across the room, such as a link or a check box
//! row: one target, and its hover fill and focus ring on that target.

use egui::{CornerRadius, Rect, Response, Sense, Ui, pos2, vec2};

use super::focus_ring;
use super::press::{Pointer, hover_fill};
use crate::theme::RADIUS;
use crate::tokens;

pub(super) struct PaintedRow {
    /// Where the row's own parts go.
    pub(super) content: Rect,
    pub(super) response: Response,
}

impl PaintedRow {
    /// The hover fill under the pointer or the keyboard's focus.
    pub(super) fn paint_fill(&self, ui: &Ui) {
        let tokens = tokens::current(ui.ctx());
        let pointer = Pointer::of_focusable(&self.response);
        let fill = hover_fill(ui, &self.response, pointer, &tokens);
        ui.painter()
            .rect_filled(self.response.rect, CornerRadius::same(RADIUS), fill);
    }

    pub(super) fn paint_ring(&self, ui: &Ui) {
        focus_ring(ui, &self.response, RADIUS);
    }
}

/// A row `height` tall across the available width. The target, and with
/// it the fill and the ring, reaches `bleed` left of the content, into
/// the margin, so the content lines up with the column around it.
pub(super) fn painted_row(ui: &mut Ui, height: f32, sense: Sense, bleed: f32) -> PaintedRow {
    let (id, content) = ui.allocate_space(vec2(ui.available_width(), height));
    let target = Rect::from_min_max(pos2(content.left() - bleed, content.top()), content.max);
    let response = ui.interact(target, id, sense);

    PaintedRow { content, response }
}
