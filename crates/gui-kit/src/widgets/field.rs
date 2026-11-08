//! The one look of every text field: the search field's filled pill.

use egui::epaint::RectShape;
use egui::layers::ShapeIdx;
use egui::{
    Align, Align2, Frame, Galley, Margin, Pos2, Rect, Response, Shape, Stroke, StrokeKind,
    TextEdit, Ui, pos2, vec2,
};

use super::focus::ring_around;
use super::line;
use crate::fonts::{BODY, Weight};
use crate::theme::{PILL_HEIGHT, PILL_RADIUS};
use crate::tokens;

/// Room between a field's round ends and its text.
pub(super) const PADDING_X: i8 = 14;
const RIM: f32 = 1.0;

/// `edit` as a filled pill [`PILL_HEIGHT`] tall, the look of the search
/// field. The caller keeps its width, alignment, hint and font.
pub fn text_field(ui: &mut Ui, edit: TextEdit<'_>) -> Response {
    field(ui, edit, None)
}

/// A [`text_field`] with `unit`, such as "s", right after its text in
/// the secondary colour, the two centred in the pill as one group. The
/// unit is only painted, so the field's accessible name has to say it.
pub fn text_field_with_unit(ui: &mut Ui, edit: TextEdit<'_>, unit: &str) -> Response {
    field(ui, edit, Some(unit))
}

fn field(ui: &mut Ui, edit: TextEdit<'_>, unit: Option<&str>) -> Response {
    let tokens = tokens::current(ui.ctx());
    let ink = if ui.is_enabled() {
        tokens.secondary
    } else {
        tokens.dim
    };

    let unit = unit.map(|unit| line(ui, unit, Weight::Regular, BODY, ink));
    let gap = tokens.sizes.text_to_icon;
    let room = unit.as_ref().map_or(0.0, |unit| unit.size().x + gap);

    // The text centres left of the middle by half the unit's room, so the
    // text and the unit after it centre as one.
    let margin = Margin {
        left: PADDING_X,
        right: PADDING_X.saturating_add(whole_points(room)),
        top: 0,
        bottom: 0,
    };

    let edit = if unit.is_some() {
        edit.align(Align2([Align::Center, Align::Center]))
    } else {
        edit
    };

    // The fill goes under the text but depends on the focus the field
    // only reports once it is drawn.
    let slot = ui.painter().add(Shape::Noop);
    let output = edit
        .frame(Frame::NONE.inner_margin(margin))
        .min_size(vec2(0.0, PILL_HEIGHT))
        .show(ui);

    let response = output.response.response;
    paint_field(ui, slot, response.rect, PILL_RADIUS, response.has_focus());

    if let Some(unit) = unit {
        let text = response.rect - margin;
        let text_end = text.center().x + output.galley.size().x / 2.0;
        let at = unit_at(&unit, response.rect, text_end + gap);
        ui.painter().galley(at, unit, ink);
    }

    response
}

/// Where `unit` goes when it starts at `left`, kept inside the pill when
/// a long text pushes it past the end.
fn unit_at(unit: &Galley, pill: Rect, left: f32) -> Pos2 {
    let last = pill.right() - f32::from(PADDING_X) - unit.size().x;
    let start = pos2(left.min(last), pill.center().y);
    Align2::LEFT_CENTER.anchor_size(start, unit.size()).min
}

/// `points` rounded up to whole points, as a margin holds them.
fn whole_points(points: f32) -> i8 {
    (0..=i8::MAX)
        .find(|&whole| f32::from(whole) >= points)
        .unwrap_or(i8::MAX)
}

/// A field's fill in `slot` on `field` with `radius` corners, a step
/// lighter while it holds the focus.
///
/// The soft fill alone does not reach 3:1 on a card, so a thin rim round
/// the whole shape carries the field's edge. A focused field shows the
/// focus ring in its place, so a field never has two frames.
pub(super) fn paint_field(ui: &Ui, slot: ShapeIdx, field: Rect, radius: u8, focused: bool) {
    let tokens = tokens::current(ui.ctx());
    let (fill, rim) = if focused {
        (tokens.hover, Stroke::NONE)
    } else {
        (tokens.field, Stroke::new(RIM, tokens.field_rim))
    };

    let shape = RectShape::new(field, radius, fill, rim, StrokeKind::Inside);
    ui.painter().set(slot, shape);
    if focused {
        ring_around(ui, field, radius);
    }
}
