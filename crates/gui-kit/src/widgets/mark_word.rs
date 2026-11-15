//! A small mark and a word after it on one line, such as a status dot
//! and "Running", one node for screen readers.

use std::sync::Arc;

use egui::{Galley, Rect, Response, Role, Sense, Ui, Vec2, WidgetInfo, pos2, vec2};

use crate::tokens;

/// `mark` sized, `gap` before `text`, both on the line's middle, named
/// `label` with `role`. `paint` draws the mark into its rect.
pub(super) fn mark_and_word(
    ui: &mut Ui,
    (mark, gap): (Vec2, f32),
    text: Arc<Galley>,
    (role, label): (Role, &str),
    paint: impl FnOnce(&Ui, Rect),
) -> Response {
    let size = vec2(mark.x + gap + text.size().x, text.size().y.max(mark.y));
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(role, true, label));

    if ui.is_rect_visible(rect) {
        let middle = rect.center().y;
        let mark_rect = Rect::from_min_size(pos2(rect.left(), middle - mark.y / 2.0), mark);
        paint(ui, mark_rect);

        let at = pos2(mark_rect.right() + gap, middle - text.size().y / 2.0);
        let ink = tokens::current(ui.ctx()).secondary;
        ui.painter().galley(at, text, ink);
    }

    response
}
