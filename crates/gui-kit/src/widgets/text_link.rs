//! Text that leads somewhere inside running text, such as a package name
//! in a list of dependencies, and the heading of a part that folds away,
//! such as a package's file list.

use egui::{Rect, Response, Role, Sense, Stroke, Ui, WidgetInfo, pos2, vec2};

use super::{focus_ring, line};
use crate::fonts::{BODY, SMALL, Weight};
use crate::theme::RADIUS;
use crate::ui_ext::icon_image;
use crate::{icons, tokens};

const CHEVRON_GAP: f32 = 6.0;
const CHEVRON: f32 = 14.0;

/// `text` in the accent text colour, underlined under the pointer and
/// focusable. `name` is what screen readers say, such as "Open gegl".
pub fn text_link(ui: &mut Ui, text: &str, name: &str) -> Response {
    inline(ui, text, name, Role::Link)
}

/// [`text_link`] that acts here, such as "+3 more" that shows the rest
/// of a list in place.
pub fn text_button(ui: &mut Ui, text: &str, name: &str) -> Response {
    inline(ui, text, name, Role::Button)
}

fn inline(ui: &mut Ui, text: &str, name: &str, role: Role) -> Response {
    let tokens = tokens::current(ui.ctx());
    let galley = line(ui, text, Weight::Medium, BODY, tokens.accent_text);
    let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(role, ui.is_enabled(), name));

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let painter = ui.painter();
    painter.galley(rect.min, galley, tokens.accent_text);
    if response.hovered() || response.has_focus() {
        let base = rect.bottom() - 1.0;
        painter.hline(rect.x_range(), base, Stroke::new(1.0, tokens.accent_text));
    }

    focus_ring(ui, &response, RADIUS);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A chevron and `title`, a button that folds the part under it.
/// Screen readers hear whether it is open.
pub fn disclosure(ui: &mut Ui, title: &str, open: bool) -> Response {
    folding(ui, title, None, open)
}

/// [`disclosure`] with a `count` after the title, dimmed as a
/// [`minor_heading`](super::minor_heading)'s.
pub fn counted_disclosure(ui: &mut Ui, title: &str, count: &str, open: bool) -> Response {
    folding(ui, title, Some(count), open)
}

fn folding(ui: &mut Ui, title: &str, count: Option<&str>, open: bool) -> Response {
    let tokens = tokens::current(ui.ctx());
    let gap = tokens.sizes.control_gap;
    let galley = line(ui, title, Weight::SemiBold, BODY, tokens.text);
    let number = count.map(|count| line(ui, count, Weight::Regular, SMALL, tokens.secondary));
    let number_width = number.as_ref().map_or(0.0, |number| gap + number.size().x);

    let size = vec2(
        CHEVRON + CHEVRON_GAP + galley.size().x + number_width,
        galley.size().y.max(CHEVRON),
    );

    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let name = count.map_or_else(|| title.to_owned(), |count| format!("{title} {count}"));
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &name));
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_expanded(open));

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let mid = rect.center().y;
    let chevron = Rect::from_center_size(
        pos2(rect.left() + CHEVRON / 2.0, mid),
        vec2(CHEVRON, CHEVRON),
    );

    let ink = if response.hovered() {
        tokens.text
    } else {
        tokens.secondary
    };

    let glyph = if open { icons::GO_DOWN } else { icons::GO_NEXT };

    icon_image(ui, glyph, ink).paint_at(ui, chevron);

    let at = pos2(chevron.right() + CHEVRON_GAP, mid - galley.size().y / 2.0);
    let title_right = at.x + galley.size().x;
    ui.painter().galley(at, galley, tokens.text);
    if let Some(number) = number {
        let count_at = pos2(title_right + gap, mid - number.size().y / 2.0);
        ui.painter().galley(count_at, number, tokens.secondary);
    }

    focus_ring(ui, &response, RADIUS);
    response
}
