//! A row across a card with a check box and its label, the whole row one
//! target. For a choice that waits for a later button, where a switch
//! would read as taking effect at once.

use egui::{
    CornerRadius, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, pos2, vec2,
};

use super::card::in_card_row;
use super::check::partial;
use super::press::{Pointer, faded};
use super::text::cut;
use super::{focus_ring, line as text_line, rich};
use crate::fonts::{BODY, SMALL, Weight};
use crate::theme::RADIUS;
use crate::tokens;

pub const CHECK_ROW_HEIGHT: f32 = 40.0;
const HEIGHT_WITH_LINE: f32 = 56.0;
/// How far the hover fill reaches left of the box, into the margin, so
/// the box lines up with the fields and text of its column.
const PADDING_X: f32 = 12.0;
const BOX: f32 = 18.0;
const BOX_RADIUS: u8 = 4;
const GAP: f32 = 12.0;

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

/// What a row says: its label, the line under it and the tooltip.
#[derive(Clone, Copy)]
struct Words<'a> {
    label: &'a str,
    line: Option<&'a str>,
    tooltip: &'a str,
}

/// A check box named `label`, `tooltip` on hover when not empty. A click
/// on an enabled row reports `clicked`, the caller flips its state.
pub fn check_row(ui: &mut Ui, on: bool, label: &str, tooltip: &str, enabled: bool) -> Response {
    let words = Words {
        label,
        line: None,
        tooltip,
    };

    show(ui, on, words, enabled)
}

pub fn check_row_with_line(
    ui: &mut Ui,
    on: bool,
    texts: CheckRowTexts<'_>,
    enabled: bool,
) -> Response {
    let words = Words {
        label: texts.label,
        line: Some(texts.line),
        tooltip: texts.full,
    };

    let response = show(ui, on, words, enabled);
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_description(texts.full));
    response
}

fn show(ui: &mut Ui, on: bool, words: Words<'_>, enabled: bool) -> Response {
    let enabled = enabled && ui.is_enabled();
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

    let width = ui.available_width();
    let (rect, mut response) = ui.allocate_exact_size(vec2(width, height), sense);
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, on, words.label));
    if !words.tooltip.is_empty() {
        response = response.on_hover_text(words.tooltip);
    }

    if ui.is_rect_visible(rect) {
        paint(ui, &response, on, words, enabled);
    }

    response
}

fn paint(ui: &Ui, response: &Response, on: bool, words: Words<'_>, enabled: bool) {
    let rect = response.rect;
    let tokens = tokens::current(ui.ctx());
    let fade = |color| faded(color, enabled);

    // A second, wider fill inside a card row's own would nest two.
    let under = Pointer::of_focusable(response);
    if enabled && under.is_under() && !in_card_row(ui) {
        let fill = under.fill(&tokens, tokens.hover);
        ui.painter()
            .rect_filled(bleed(rect), CornerRadius::same(RADIUS), fill);
    }

    let mid = rect.center().y;
    let square = Rect::from_center_size(pos2(rect.left() + BOX / 2.0, mid), vec2(BOX, BOX));

    let painter = ui.painter();
    let corner = CornerRadius::same(BOX_RADIUS);
    if on {
        painter.rect_filled(square, corner, fade(tokens.accent));
        let mark = partial(square.shrink(2.0), 1.0);
        painter.line(mark, Stroke::new(2.0, fade(tokens.on_accent)));
    } else {
        painter.rect_stroke(
            square,
            corner,
            Stroke::new(1.5, fade(tokens.secondary)),
            StrokeKind::Inside,
        );
    }

    let galley = text_line(ui, words.label, Weight::Medium, BODY, fade(tokens.text));
    let text_left = square.right() + GAP;
    if let Some(line) = words.line {
        let room = (rect.right() - text_left).max(0.0);
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

    focus_ring(ui, response, RADIUS);
}

fn bleed(rect: Rect) -> Rect {
    Rect::from_min_max(pos2(rect.left() - PADDING_X, rect.top()), rect.max)
}
