use egui::{CornerRadius, Rect, Response, Role, Sense, StrokeKind, Ui, WidgetInfo, pos2, vec2};

use super::card::light_rim;
use super::icon_tile::{TileSize, paint_icon};
use super::list::text_line_height;
use super::press::Pointer;
use super::rich;
use super::text::{cut, cut_lines};
use crate::fonts::{BODY, SMALL, Weight};
use crate::icons::{self, Icon};
use crate::theme::CARD_RADIUS;
use crate::tokens::{self, Tokens};
use crate::ui_ext::icon_image;

/// A larger desktop font grows a tile past this, never shrinks it.
const HEIGHT: f32 = 76.0;
const GAP: f32 = 14.0;
const CHEVRON: f32 = 16.0;
const LEADING: f32 = 2.0;
const SUBTITLE_LINES: u8 = 2;

pub fn link_tile(ui: &mut Ui, icon: Icon, title: &str, subtitle: &str, width: f32) -> Response {
    let tokens = tokens::current(ui.ctx());
    let size = vec2(width, height(ui, &tokens));
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Link, ui.is_enabled(), title));
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_description(subtitle));

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let pointer = Pointer::of_keyboard(ui, &response);
    let hovered = pointer.is_under();
    let fill = pointer.fill(&tokens, tokens.card_glass);
    let rim = light_rim(tokens.card_edge);

    ui.painter().rect(
        rect,
        CornerRadius::same(CARD_RADIUS),
        fill,
        rim,
        StrokeKind::Inside,
    );

    // The same inset as a card's rows, so a tile's icon lines up with
    // theirs in one column.
    let padding = tokens.sizes.row_padding.x;
    let edge = TileSize::Row.edge();
    let badge = Rect::from_center_size(
        pos2(rect.left() + padding + edge / 2.0, rect.center().y),
        vec2(edge, edge),
    );

    paint_icon(ui, badge.center(), TileSize::Row, icon.for_row());
    let text = Rect::from_x_y_ranges(
        badge.right() + GAP..=rect.right() - padding - CHEVRON - GAP,
        rect.y_range(),
    );

    if paint_text(ui, &tokens, text, title, subtitle) {
        response = response.on_hover_ui(|tip| {
            tip.label(title);
            tip.label(subtitle);
        });
    }

    let chevron = Rect::from_center_size(
        pos2(rect.right() - padding - CHEVRON / 2.0, rect.center().y),
        vec2(CHEVRON, CHEVRON),
    );

    let ink = if hovered { tokens.text } else { tokens.dim };
    icon_image(ui, icons::GO_NEXT, ink).paint_at(ui, chevron);

    response
}

/// Room for the title and [`SUBTITLE_LINES`] of subtitle whatever the
/// subtitle says, so the tiles of one line are as tall.
fn height(ui: &Ui, tokens: &Tokens) -> f32 {
    let title = text_line_height(ui, Weight::SemiBold, BODY);
    let subtitle = text_line_height(ui, Weight::Regular, SMALL);
    let text = subtitle.mul_add(f32::from(SUBTITLE_LINES), title + LEADING);
    tokens.sizes.row_padding.y.mul_add(2.0, text).max(HEIGHT)
}

/// Returns whether either was cut.
fn paint_text(ui: &Ui, tokens: &Tokens, area: Rect, title: &str, subtitle: &str) -> bool {
    let width = area.width().max(0.0);
    let title = cut(
        ui,
        rich(ui, title, Weight::SemiBold, BODY, tokens.text),
        width,
    );

    let subtitle = cut_lines(
        ui,
        rich(ui, subtitle, Weight::Regular, SMALL, tokens.secondary),
        width,
        usize::from(SUBTITLE_LINES),
    );

    let was_cut = title.elided || subtitle.elided;
    let title_height = title.size().y;
    let top = area.center().y - (title_height + LEADING + subtitle.size().y) / 2.0;
    let painter = ui.painter();
    painter.galley(pos2(area.left(), top), title, tokens.text);
    painter.galley(
        pos2(area.left(), top + title_height + LEADING),
        subtitle,
        tokens.secondary,
    );

    was_cut
}
