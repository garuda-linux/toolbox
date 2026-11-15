//! The sheet's top edge over the page: a hairline and the popup shadow
//! cast upward, so the page's text under the sheet does not read as cut.

use egui::{CornerRadius, Rect, Stroke, StrokeKind, Ui, pos2};
use toolbox_gui_kit::tokens;

/// The shadow over the page above the sheet at `rect`, inside `room`. It
/// stays above the sheet, as under the glass it would darken it. Drawn
/// before the sheet clips its layer to itself.
pub(super) fn shadow(ui: &Ui, room: Rect, rect: Rect, corners: CornerRadius) {
    let mut shadow = ui.visuals().popup_shadow;
    shadow.offset[1] = shadow.offset[1].saturating_abs().saturating_neg();
    let above = Rect::from_min_max(room.left_top(), pos2(room.right(), rect.top()));
    ui.painter()
        .with_clip_rect(above)
        .add(shadow.as_shape(rect, corners));
}

/// The hairline along the sheet's top and round its upper corners, down to
/// where the corners meet its sides, which lie on the window's and the
/// sidebar's edges.
pub(super) fn top_edge(ui: &Ui, rect: Rect, corners: CornerRadius) {
    let outline = Stroke::new(1.0, tokens::current(ui.ctx()).outline);
    let round = f32::from(corners.nw.max(corners.ne)).max(1.0);
    let band = Rect::from_min_max(rect.min, pos2(rect.right(), rect.top() + round));
    ui.painter()
        .with_clip_rect(band)
        .rect_stroke(rect, corners, outline, StrokeKind::Inside);
}
