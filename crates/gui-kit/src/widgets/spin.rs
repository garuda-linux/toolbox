//! A whole number with its unit in one input, stepped by the up and down
//! arrows at its right end, as Breeze's spin box draws them: a column of
//! two arrows inside the frame.

#[cfg(test)]
mod tests;

use std::ops::RangeInclusive;

use egui::accesskit::Role;
use egui::{Id, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, WidgetInfo, pos2, vec2};

use super::field::{INSET, field_height, paint_field};
use super::focus::focus_from_keyboard;
use super::number::inset_number_with_unit;
use crate::theme::FIELD_RADIUS;
use crate::tokens::{self, mix};

/// Darkly's `SpinBox_ArrowButtonWidth`, the column of arrows.
const ARROW_COLUMN: f32 = 20.0;
/// Darkly's `ArrowSize`, the side of an arrow's square.
const ARROW_SIZE: f32 = 10.0;
/// Darkly's `arrowShade`, how far an arrow moves from the text toward the
/// base (darklyhelper.cpp:56).
const ARROW_SHADE: f32 = 0.15;
/// Breeze's `PenWidth::Symbol`, half of it as the arrow's inset.
const SYMBOL_HALF_PEN: f32 = 0.5005;

/// The words of a [`spin_field`]. `name` names the number for screen
/// readers and should say the unit, which they do not hear. `less` and
/// `more` name the down and up arrows.
#[derive(Clone, Copy, Debug)]
pub struct SpinTexts<'a> {
    /// Tells the field apart from the other fields of its ui.
    pub salt: &'a str,
    pub name: &'a str,
    pub unit: &'a str,
    pub less: &'a str,
    pub more: &'a str,
}

#[derive(Clone, Copy)]
enum Step {
    Less,
    More,
}

/// A field for `value`, `width` wide, held to `range`. A typed number
/// counts as in a number field. Returns whether `value` changed.
pub fn spin_field(
    ui: &mut Ui,
    value: &mut u32,
    range: RangeInclusive<u32>,
    width: f32,
    texts: SpinTexts<'_>,
) -> bool {
    let before = *value;
    let (frame, _) = ui.allocate_exact_size(vec2(width, field_height(ui)), Sense::hover());

    // The frame goes under every part but follows the number's focus,
    // known only once it is drawn.
    let slot = ui.painter().add(Shape::Noop);
    let id = ui.scope_id().with(("spin_field", texts.salt));

    let inner = frame.shrink(INSET);
    let column = Rect::from_min_max(
        pos2(inner.right() - ARROW_COLUMN, inner.top()),
        inner.right_bottom(),
    );

    let half = ARROW_COLUMN / 2.0;
    let up = Rect::from_min_size(
        pos2(column.left(), column.center().y - half),
        vec2(ARROW_COLUMN, half),
    );

    let down = up.translate(vec2(0.0, half));
    let text = Rect::from_min_max(inner.min, pos2(column.left(), inner.bottom()));

    let (least, most) = (*range.start(), *range.end());
    if stepper(
        ui,
        id.with("less"),
        down,
        Step::Less,
        (texts.less, *value <= least),
    ) {
        *value = value.saturating_sub(1);
    }

    let number = ui.put(text, |ui: &mut Ui| {
        inset_number_with_unit(
            ui,
            value,
            range.clone(),
            text.width(),
            (texts.salt, texts.name, texts.unit),
        )
    });

    if stepper(
        ui,
        id.with("more"),
        up,
        Step::More,
        (texts.more, *value >= most),
    ) {
        *value = value.saturating_add(1);
    }

    *value = (*value).clamp(*range.start(), *range.end());
    paint_field(ui, slot, frame, FIELD_RADIUS, number.has_focus());

    *value != before
}

/// One arrow of the column, named and at its limit or not. It is a click
/// target with no fill of its own.
fn stepper(ui: &Ui, id: Id, rect: Rect, step: Step, (name, at_limit): (&str, bool)) -> bool {
    let response = ui.interact(rect, id, Sense::click());
    let enabled = ui.is_enabled() && !at_limit;
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, name));

    if ui.is_rect_visible(rect) {
        paint_arrow(ui, rect, &response, step, at_limit);
    }

    response.clicked()
}

/// Darkly's `renderSpinBoxArrow`: a polyline in the arrow's square, centred
/// in its rect. The text shaded toward the view colour, the decoration
/// focus under the pointer or with the focus from a key, and the dim text
/// at the limit, where an arrow does not react to either.
fn paint_arrow(ui: &Ui, rect: Rect, response: &Response, step: Step, at_limit: bool) {
    let tokens = tokens::current(ui.ctx());
    let ink = if at_limit {
        tokens.dim
    } else if response.hovered() || focus_from_keyboard(ui, response) {
        tokens.decoration_focus
    } else {
        mix(tokens.text, tokens.view, ARROW_SHADE)
    };

    // Breeze's points in the square for a size of 10 px, where its
    // `sizeOffset` is half a pixel.
    let inset = SYMBOL_HALF_PEN;
    let offset = 0.5;
    let points: [Pos2; 3] = match step {
        Step::More => [
            pos2(inset, 7.5 - inset - offset),
            pos2(ARROW_SIZE / 2.0, 2.5 - offset),
            pos2(ARROW_SIZE - inset, 7.5 - inset - offset),
        ],
        Step::Less => [
            pos2(inset, 2.5 + inset + offset),
            pos2(ARROW_SIZE / 2.0, 7.5 + offset),
            pos2(ARROW_SIZE - inset, 2.5 + inset + offset),
        ],
    };

    let origin = rect.center() - vec2(ARROW_SIZE, ARROW_SIZE) / 2.0;
    let line: Vec<Pos2> = points
        .into_iter()
        .map(|point| origin + point.to_vec2())
        .collect();

    ui.painter().line(line, Stroke::new(1.0, ink));
}
