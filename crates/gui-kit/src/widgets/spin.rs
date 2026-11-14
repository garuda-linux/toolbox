//! A whole number with its unit between a step down and a step up, all
//! inside one filled pill, so the three read as one control.

#[cfg(test)]
mod tests;

use std::ops::RangeInclusive;

use egui::accesskit::Role;
use egui::{Align2, Id, Rect, Response, Sense, Shape, Ui, WidgetInfo, pos2, vec2};

use super::field::paint_field;
use super::focus::focus_ring;
use super::number::inset_number_with_unit;
use super::press::{Pointer, faded};
use super::text::line;
use crate::fonts::{BODY, Weight};
use crate::theme::{PILL_HEIGHT, PILL_RADIUS};
use crate::tokens;

/// The words of a [`spin_field`]. `name` names the number for screen
/// readers and should say the unit, which they do not hear. `less` and
/// `more` name the steppers.
#[derive(Clone, Copy, Debug)]
pub struct SpinTexts<'a> {
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
    let (pill, _) = ui.allocate_exact_size(vec2(width, PILL_HEIGHT), Sense::hover());

    // The fill goes under every part but follows the number's focus,
    // known only once it is drawn.
    let slot = ui.painter().add(Shape::Noop);
    let id = ui.scope_id().with(("spin_field", texts.name));
    let less = Rect::from_min_size(pill.min, vec2(PILL_HEIGHT, PILL_HEIGHT));
    let more = Rect::from_min_size(
        pos2(pill.right() - PILL_HEIGHT, pill.top()),
        vec2(PILL_HEIGHT, PILL_HEIGHT),
    );

    let middle = Rect::from_min_max(
        pos2(less.right(), pill.top()),
        pos2(more.left(), pill.bottom()),
    );

    if stepper(ui, id.with("less"), less, Step::Less, texts.less) {
        *value = value.saturating_sub(1);
    }

    let number = ui.put(middle, |ui: &mut Ui| {
        inset_number_with_unit(
            ui,
            value,
            range.clone(),
            middle.width(),
            (texts.name, texts.unit),
        )
    });

    if stepper(ui, id.with("more"), more, Step::More, texts.more) {
        *value = value.saturating_add(1);
    }

    *value = (*value).clamp(*range.start(), *range.end());
    paint_field(ui, slot, pill, PILL_RADIUS, number.has_focus());

    *value != before
}

/// A round button inside the pill. A circle of the hover fill marks the
/// pointer and the focus.
fn stepper(ui: &Ui, id: Id, rect: Rect, step: Step, name: &str) -> bool {
    let response = ui.interact(rect, id, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), name));

    if ui.is_rect_visible(rect) {
        paint_stepper(ui, rect, &response, step);
    }

    response.clicked()
}

fn paint_stepper(ui: &Ui, rect: Rect, response: &Response, step: Step) {
    let tokens = tokens::current(ui.ctx());
    let pointer = Pointer::of_focusable(response);
    if pointer.is_under() {
        let fill = pointer.fill(&tokens, tokens.field);
        ui.painter()
            .circle_filled(rect.center(), f32::from(PILL_RADIUS), fill);
    }

    let symbol = match step {
        Step::Less => "−",
        Step::More => "+",
    };

    let ink = faded(tokens.text, ui.is_enabled());
    let glyph = line(ui, symbol, Weight::SemiBold, BODY, ink);
    let at = Align2::CENTER_CENTER
        .anchor_size(rect.center(), glyph.size())
        .min;

    ui.painter().galley(at, glyph, ink);
    focus_ring(ui, response, PILL_RADIUS);
}
