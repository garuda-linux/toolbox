use egui::{CornerRadius, Rect, Response, Role, Sense, StrokeKind, Ui, WidgetInfo, pos2, vec2};

use super::card::light_rim;
use super::icon_tile::{TileSize, page_tile, paint_icon_tile};
use super::press::Pointer;
use super::text::cut;
use super::{focus_ring, rich};
use crate::fonts::{BODY, SMALL, Weight};
use crate::icons::{self, Icon};
use crate::theme::CARD_RADIUS;
use crate::tokens::{self, Tokens};
use crate::ui_ext::icon_image;

const HEIGHT: f32 = 76.0;
const PADDING: f32 = 18.0;
const GAP: f32 = 14.0;
const CHEVRON: f32 = 16.0;
const LEADING: f32 = 2.0;

pub fn link_tile(ui: &mut Ui, icon: Icon, title: &str, subtitle: &str, width: f32) -> Response {
    let tokens = tokens::current(ui.ctx());
    let (rect, mut response) = ui.allocate_exact_size(vec2(width, HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Link, ui.is_enabled(), title));
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_description(subtitle));

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let pointer = Pointer::of_focusable(&response);
    let hovered = pointer.is_under();
    let fill = pointer.fill(&tokens, tokens.card);
    let rim = light_rim(ui, tokens.outline);

    ui.painter().rect(
        rect,
        CornerRadius::same(CARD_RADIUS),
        fill,
        rim,
        StrokeKind::Inside,
    );

    let edge = TileSize::Row.edge(ui);
    let badge = Rect::from_center_size(
        pos2(rect.left() + PADDING + edge / 2.0, rect.center().y),
        vec2(edge, edge),
    );

    paint_icon_tile(ui, badge.center(), TileSize::Row, icon, page_tile(ui.ctx()));
    let text = Rect::from_x_y_ranges(
        badge.right() + GAP..=rect.right() - PADDING - CHEVRON - GAP,
        rect.y_range(),
    );

    if paint_text(ui, &tokens, text, title, subtitle) {
        response = response.on_hover_ui(|tip| {
            tip.label(title);
            tip.label(subtitle);
        });
    }

    let chevron = Rect::from_center_size(
        pos2(rect.right() - PADDING - CHEVRON / 2.0, rect.center().y),
        vec2(CHEVRON, CHEVRON),
    );

    let ink = if hovered { tokens.text } else { tokens.dim };
    icon_image(ui, icons::GO_NEXT, ink).paint_at(ui, chevron);
    focus_ring(ui, &response, CARD_RADIUS);

    response
}

/// Returns whether either was cut.
fn paint_text(ui: &Ui, tokens: &Tokens, area: Rect, title: &str, subtitle: &str) -> bool {
    let width = area.width().max(0.0);
    let title = cut(
        ui,
        rich(ui, title, Weight::SemiBold, BODY, tokens.text),
        width,
    );

    let subtitle = cut(
        ui,
        rich(ui, subtitle, Weight::Regular, SMALL, tokens.secondary),
        width,
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
