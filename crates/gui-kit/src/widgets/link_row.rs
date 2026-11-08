//! A compact row that leads somewhere, for a card of links such as About's
//! Help: an icon, a title and a chevron, the whole row one target.

use egui::{CornerRadius, Rect, Response, Role, Sense, Ui, WidgetInfo, pos2, vec2};

use super::icon_tile::{TileSize, page_tile, paint_icon_tile};
use super::press::Pointer;
use super::{focus_ring, line};
use crate::fonts::{BODY, Weight};
use crate::icons::{self, Icon};
use crate::theme::RADIUS;
use crate::tokens;
use crate::ui_ext::icon_image;

const HEIGHT: f32 = 44.0;
const PADDING_X: f32 = 12.0;
const GAP: f32 = 12.0;
const CHEVRON: f32 = 14.0;

/// A row across the available width. `role` is [`Role::Link`] for one that
/// opens a page, [`Role::Button`] for one that acts here.
pub fn link_row(ui: &mut Ui, icon: Icon, title: &str, role: Role) -> Response {
    let tokens = tokens::current(ui.ctx());
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(role, ui.is_enabled(), title));

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let pointer = Pointer::of_focusable(&response);
    if pointer.is_under() {
        let fill = pointer.fill(&tokens, tokens.hover);
        ui.painter()
            .rect_filled(rect, CornerRadius::same(RADIUS), fill);
    }

    let mid = rect.center().y;
    let edge = TileSize::Line.edge(ui);
    let square = Rect::from_center_size(
        pos2(rect.left() + PADDING_X + edge / 2.0, mid),
        vec2(edge, edge),
    );

    paint_icon_tile(
        ui,
        square.center(),
        TileSize::Line,
        icon,
        page_tile(ui.ctx()),
    );

    let galley = line(ui, title, Weight::Medium, BODY, tokens.text);
    let text_left = square.right() + GAP;
    ui.painter().galley(
        pos2(text_left, mid - galley.size().y / 2.0),
        galley,
        tokens.text,
    );

    let chevron = Rect::from_center_size(
        pos2(rect.right() - PADDING_X - CHEVRON / 2.0, mid),
        vec2(CHEVRON, CHEVRON),
    );

    icon_image(ui, icons::GO_NEXT, tokens.secondary).paint_at(ui, chevron);
    focus_ring(ui, &response, RADIUS);
    response
}
