//! A row across a card with a check box and its label, the whole row one
//! target. For a choice that waits for a later button, where a switch
//! would read as taking effect at once.

use egui::{Rect, Response, Role, Sense, Shadow, Stroke, StrokeKind, Ui, WidgetInfo, pos2, vec2};

use super::card::in_card_row;
use super::check::{box_fill, mark_shadow, partial};
use super::painted_row::{PaintedRow, painted_row};
use super::press::{Pointer, faded};
use super::rich;
use super::text::cut;
use super::top_light::paint_top_light;
use crate::fonts::{BODY, SMALL, Weight};
use crate::theme::CHECK_RADIUS;
use crate::tokens;

pub const CHECK_ROW_HEIGHT: f32 = 40.0;
const HEIGHT_WITH_LINE: f32 = 56.0;
/// How far the target, its hover fill and its focus fill reach left of the box,
/// into the margin, so the box lines up with the fields and text of its
/// column.
const PADDING_X: f32 = 12.0;
/// Darkly's `CheckBox_Size`, the slot the box and its mark fill.
pub(super) const BOX: f32 = 18.0;
/// The box sits 1 px into the slot on each side, as Darkly's frame does.
pub(super) const SQUARE: f32 = BOX - 2.0;
/// Breeze's `CheckBox_ItemSpacing`, between the slot and the label.
const GAP: f32 = 4.0;

/// The texts of a [`check_row_with_line`].
#[derive(Clone, Copy, Debug)]
pub struct CheckRowTexts<'a> {
    pub label: &'a str,
    /// Under the label in secondary text, cut with "…" when too long.
    pub line: &'a str,
    /// The line's whole text, for the tooltip and the accessible
    /// description.
    pub full: &'a str,
}

#[derive(Clone, Copy)]
struct Words<'a> {
    label: &'a str,
    line: Option<&'a str>,
    tooltip: &'a str,
}

/// A check box labelled `label`, with `tooltip` shown on hover when not empty.
pub fn check_row(ui: &mut Ui, on: bool, label: &str, tooltip: &str) -> Response {
    let words = Words {
        label,
        line: None,
        tooltip,
    };

    show(ui, on, words)
}

pub fn check_row_with_line(ui: &mut Ui, on: bool, texts: CheckRowTexts<'_>) -> Response {
    let words = Words {
        label: texts.label,
        line: Some(texts.line),
        tooltip: texts.full,
    };

    let response = show(ui, on, words);
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_description(texts.full));
    response
}

fn show(ui: &mut Ui, on: bool, words: Words<'_>) -> Response {
    let enabled = ui.is_enabled();
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };

    let height = if words.line.is_some() {
        HEIGHT_WITH_LINE
    } else {
        CHECK_ROW_HEIGHT
    };

    // Outside a card row the fill reaches into the margin, so the box
    // lines up with the column. A card row paints its own fill.
    let inside = in_card_row(ui);
    let bleed = if inside { 0.0 } else { PADDING_X };
    let row = painted_row(ui, height, sense, bleed);
    let mut response = row.response.clone();
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, on, words.label));
    if !words.tooltip.is_empty() {
        response = response.on_hover_text(words.tooltip);
    }

    // A second, wider fill inside a card row's own would nest two.
    let lit = enabled && !inside;
    let was_cut =
        ui.is_rect_visible(row.response.rect) && paint(ui, &row, on, words, (enabled, lit));

    // A row with its own tooltip says more than the label already.
    if was_cut && words.tooltip.is_empty() {
        response = response.on_hover_text(words.label);
    }

    response
}

/// Draws the row. Returns whether the label was cut to fit.
fn paint(
    ui: &Ui,
    row: &PaintedRow,
    on: bool,
    words: Words<'_>,
    (enabled, lit): (bool, bool),
) -> bool {
    let rect = row.content;
    let tokens = tokens::current(ui.ctx());
    let fade = |color| faded(ui, color, enabled);
    if lit {
        row.paint_fill(ui);
    }

    let mid = rect.center().y;
    let slot = Rect::from_center_size(pos2(rect.left() + BOX / 2.0, mid), vec2(BOX, BOX));
    let fill = box_fill(&tokens, on, Pointer::of_keyboard(ui, &row.response));
    let painter = ui.painter();
    let square = Rect::from_center_size(slot.center(), vec2(SQUARE, SQUARE));
    let shadow = Shadow {
        offset: [0, 1],
        blur: if on { 4 } else { 2 },
        spread: 0,
        color: fade(tokens.control_shadow),
    };

    painter.add(shadow.as_shape(square, CHECK_RADIUS));
    painter.rect_filled(square, CHECK_RADIUS, fade(fill));
    if !on {
        painter.rect_stroke(
            square,
            CHECK_RADIUS,
            Stroke::new(1.0, fade(tokens.card_outline)),
            StrokeKind::Inside,
        );
    }

    paint_top_light(painter, square, CHECK_RADIUS);

    if on {
        let mark = partial(square, 1.0);
        let under = partial(square.translate(vec2(0.0, 1.0)), 1.0);
        painter.line(under, Stroke::new(2.0, fade(mark_shadow(&tokens))));
        painter.line(mark, Stroke::new(2.0, fade(tokens.highlighted_text)));
    }

    let text_left = slot.right() + GAP;
    let room = (rect.right() - text_left).max(0.0);
    let label = rich(ui, words.label, Weight::Medium, BODY, fade(tokens.text));
    let galley = cut(ui, label, room);
    let was_cut = galley.elided;
    if let Some(line) = words.line {
        let detail = cut(
            ui,
            rich(ui, line, Weight::Regular, SMALL, fade(tokens.secondary)),
            room,
        );

        let gap = tokens.sizes.tight_gap;
        let top = mid - (galley.size().y + gap + detail.size().y) / 2.0;
        let below = top + galley.size().y + gap;
        painter.galley(pos2(text_left, top), galley, tokens.text);
        painter.galley(pos2(text_left, below), detail, tokens.secondary);
    } else {
        painter.galley(
            pos2(text_left, mid - galley.size().y / 2.0),
            galley,
            tokens.text,
        );
    }

    was_cut
}
