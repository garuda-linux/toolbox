//! The rows of an open dropdown list and the colour strips before
//! their names.

use egui::{
    Align, Color32, CornerRadius, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo,
    pos2, vec2,
};

use super::super::menu::{ITEM_INSET, LIST_GAP};
use super::super::press::{Pointer, hover_amount};
use super::super::text::cut;
use super::super::{focus_ring, mark_selected, rich};
use super::OPTION_HEIGHT;
use crate::fonts::{BODY, Weight};
use crate::theme::RADIUS;
use crate::tokens;
use crate::tokens::Tokens;

/// A colour strip before an option's name: three columns, as wide as
/// tall per column, with the gap to the name.
pub(super) const SWATCH_WIDTH: f32 = 18.0;
const SWATCH_HEIGHT: f32 = 12.0;
pub(super) const SWATCH_GAP: f32 = 8.0;

/// The rows of an open list. `reveal` scrolls the chosen one into view.
pub(super) fn options_in(
    list: &mut Ui,
    options: &[&str],
    swatches: &[[Color32; 3]],
    selected: Option<usize>,
    reveal: bool,
) -> Option<usize> {
    list.spacing_mut().item_spacing.y = LIST_GAP;
    let mut picked = None;
    for (index, option) in options.iter().enumerate() {
        let chosen = Some(index) == selected;
        let row = option_row(list, option, chosen, swatches.get(index).copied());
        if chosen && reveal {
            row.scroll_to_me(Some(Align::Center));
        }

        if row.clicked() {
            picked = Some(index);
        }
    }

    // The close behaviour only hears the pointer. A pick from the
    // keyboard or a screen reader closes the list here.
    if picked.is_some() {
        list.close();
    }

    picked
}

/// An option's fill: the chosen one is tinted at rest, any one shows the
/// hover and press under the pointer.
pub(in crate::widgets) fn option_fill(
    tokens: &Tokens,
    chosen: bool,
    pointer: Pointer,
    hover: f32,
) -> Color32 {
    if pointer == Pointer::Pressed {
        return tokens.pressed;
    }

    let rest = if chosen {
        tokens.accent_soft
    } else {
        Color32::TRANSPARENT
    };

    rest.lerp_to_gamma(tokens.hover, hover)
}

/// The room the colour strips take before each name, none without strips.
pub(super) fn swatch_room(swatches: &[[Color32; 3]]) -> f32 {
    if swatches.is_empty() {
        0.0
    } else {
        SWATCH_WIDTH + SWATCH_GAP
    }
}

/// Three colours side by side, outlined so a transparent one keeps its
/// place. `left` and `centre_y` place the strip.
pub(super) fn paint_swatch(ui: &Ui, left: f32, centre_y: f32, colours: [Color32; 3]) {
    let tokens = tokens::current(ui.ctx());
    let rect = Rect::from_min_size(
        pos2(left, centre_y - SWATCH_HEIGHT / 2.0),
        vec2(SWATCH_WIDTH, SWATCH_HEIGHT),
    );

    let column = SWATCH_WIDTH / 3.0;
    for (step, colour) in [0.0_f32, 1.0, 2.0].into_iter().zip(colours) {
        let part = Rect::from_min_size(
            pos2(step.mul_add(column, rect.left()), rect.top()),
            vec2(column, rect.height()),
        );

        ui.painter().rect_filled(part, 0.0, colour);
    }

    ui.painter().rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0, tokens.outline),
        StrokeKind::Inside,
    );
}

/// Too wide text is cut, shown whole on hover and to screen readers.
fn option_row(ui: &mut Ui, text: &str, chosen: bool, swatch: Option<[Color32; 3]>) -> Response {
    let tokens = tokens::current(ui.ctx());
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, OPTION_HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::ListBoxOption, true, text));
    mark_selected(ui, &response, chosen);

    let pointer = Pointer::of(&response);
    let fill = option_fill(
        &tokens,
        chosen,
        pointer,
        hover_amount(ui, &response, pointer),
    );

    ui.painter()
        .rect_filled(rect, CornerRadius::same(RADIUS), fill);

    let weight = if chosen {
        Weight::SemiBold
    } else {
        Weight::Regular
    };

    let before = swatch.map_or(0.0, |colours| {
        paint_swatch(ui, rect.left() + ITEM_INSET, rect.center().y, colours);
        SWATCH_WIDTH + SWATCH_GAP
    });

    let room = ITEM_INSET.mul_add(-2.0, rect.width() - before).max(0.0);
    let galley = cut(ui, rich(ui, text, weight, BODY, tokens.text), room);
    let was_cut = galley.elided;
    let position = pos2(
        rect.left() + ITEM_INSET + before,
        rect.center().y - galley.size().y / 2.0,
    );

    ui.painter().galley(position, galley, tokens.text);
    focus_ring(ui, &response, RADIUS);

    if was_cut {
        response.on_hover_text(text)
    } else {
        response
    }
}
