//! A compact row that leads somewhere, for a card of links such as About's
//! Help: an icon, a title and, for a row that leads to a page, a chevron. The
//! whole row is one target.

use egui::{Rect, Response, Role, Sense, Ui, WidgetInfo, pos2, vec2};

use super::icon_tile::{ICON_EDGE, paint_icon};
use super::list::text_line_height;
use super::painted_row::painted_row;
use super::rich;
use super::text::cut;
use crate::fonts::{BODY, Weight};
use crate::icons::{self, Icon};
use crate::tokens;
use crate::ui_ext::icon_image;

/// Darkly's item rows: the text line with 3 points above and below.
const ITEM_PADDING_Y: f32 = 3.0;
const PADDING_X: f32 = 12.0;
const GAP: f32 = 12.0;
const CHEVRON: f32 = 14.0;

/// A row across the available width.
///
/// `role` is [`Role::Link`] for one that opens a page, with a chevron, and
/// [`Role::Button`] for one that acts here, without it. It is at least
/// `least` tall, for rows that share a card's height with the rows of a card
/// beside it.
pub fn link_row_at_least(ui: &mut Ui, icon: Icon, title: &str, role: Role, least: f32) -> Response {
    let tokens = tokens::current(ui.ctx());
    let height = ITEM_PADDING_Y
        .mul_add(2.0, text_line_height(ui, Weight::Medium, BODY))
        .max(least);

    let row = painted_row(ui, height, Sense::click(), 0.0);
    let (rect, response) = (row.content, row.response.clone());
    response.widget_info(|| WidgetInfo::labeled(role, ui.is_enabled(), title));

    if !ui.is_rect_visible(rect) {
        return response;
    }

    row.paint_fill(ui);
    let mid = rect.center().y;
    let edge = ICON_EDGE;
    let square = Rect::from_center_size(
        pos2(rect.left() + PADDING_X + edge / 2.0, mid),
        vec2(edge, edge),
    );

    paint_icon(ui, square.center(), icon.for_row());

    let leads = role == Role::Link;
    let chevron = Rect::from_center_size(
        pos2(rect.right() - PADDING_X - CHEVRON / 2.0, mid),
        vec2(CHEVRON, CHEVRON),
    );

    let text_left = square.right() + GAP;
    let end = if leads {
        chevron.left() - GAP
    } else {
        rect.right() - PADDING_X
    };

    let room = (end - text_left).max(0.0);
    let galley = cut(ui, rich(ui, title, Weight::Medium, BODY, tokens.text), room);
    let was_cut = galley.elided;

    ui.painter().galley(
        pos2(text_left, mid - galley.size().y / 2.0),
        galley,
        tokens.text,
    );

    if leads {
        icon_image(ui, icons::GO_NEXT, tokens.secondary).paint_at(ui, chevron);
    }

    row.paint_ring(ui);

    // The title is whole for screen readers already, the tooltip shows
    // it whole to the eye.
    if was_cut {
        response.on_hover_text(title)
    } else {
        response
    }
}
