//! A ⋯ button that opens a menu of actions on a row, and the items of
//! such a menu.

use egui::{
    CornerRadius, Popup, Rect, Response, Role, Sense, TextStyle, Ui, WidgetInfo, pos2, vec2,
};

use super::press::Pointer;
use super::{focus_ring, icon_button, line};
use crate::fonts::{BODY, SMALL, Weight};
use crate::frost::Glass;
use crate::icons::{self, Icon};
use crate::theme::{POPOVER_RADIUS, RADIUS};
use crate::tokens;
use crate::ui_ext::icon_image;

/// A row of a menu or a dropdown's list.
pub(super) const ITEM_HEIGHT: f32 = 32.0;
pub(super) const MENU_WIDTH: f32 = 220.0;
/// From an item's edge to its icon or text.
pub(super) const ITEM_INSET: f32 = 10.0;
/// Between the items of a menu or a list.
pub(super) const LIST_GAP: f32 = 2.0;

pub fn menu_button(ui: &mut Ui, alt: &str, items: impl FnOnce(&mut Ui)) -> Response {
    let button = icon_button(ui, icons::OVERFLOW_MENU, alt);
    menu_for(ui, &button, false, items);
    button
}

pub(super) fn menu_for(ui: &Ui, button: &Response, open: bool, items: impl FnOnce(&mut Ui)) {
    if open {
        Popup::open_id(ui.ctx(), Popup::default_response_id(button));
    }

    let glass = menu_glass(ui);
    Popup::menu(button)
        .width(MENU_WIDTH)
        .frame(glass.frame())
        .show(|popup| menu_items(glass, popup, items));
}

/// `items` on `glass`, a list gap apart.
pub(super) fn menu_items(glass: Glass, popup: &mut Ui, items: impl FnOnce(&mut Ui)) {
    glass.inside(popup, |list| {
        list.spacing_mut().item_spacing.y = LIST_GAP;
        items(list);
    });
}

/// The frosted panel every menu and list sits on.
pub(super) fn menu_glass(ui: &Ui) -> Glass {
    let padding = tokens::current(ui.ctx()).sizes.list_padding;
    Glass::raised(ui, CornerRadius::same(POPOVER_RADIUS), padding)
}

/// One action of a menu. A clicked item closes the menu.
pub fn menu_item(ui: &mut Ui, text: &str) -> Response {
    item(ui, None, text, false)
}

/// One action of a menu with its icon before the text, or room for one
/// when `icon` is `None`, so the texts of a menu line up.
pub fn menu_action(ui: &mut Ui, icon: Option<Icon>, text: &str) -> Response {
    item(ui, icon, text, true)
}

fn item(ui: &mut Ui, icon: Option<Icon>, text: &str, room: bool) -> Response {
    let tokens = tokens::current(ui.ctx());
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, ITEM_HEIGHT), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::MenuItem, ui.is_enabled(), text));

    let pointer = Pointer::of(&response);
    if pointer.is_under() && ui.is_enabled() {
        let fill = pointer.fill(&tokens, tokens.hover);
        ui.painter()
            .rect_filled(rect, CornerRadius::same(RADIUS), fill);
    }

    let ink = if ui.is_enabled() {
        tokens.text
    } else {
        tokens.dim
    };

    let mut left = rect.left() + ITEM_INSET;
    if room {
        let side = ui.text_style_height(&TextStyle::Body);
        let at = Rect::from_min_size(pos2(left, rect.center().y - side / 2.0), vec2(side, side));
        if let Some(icon) = icon {
            icon_image(ui, icon, ink).paint_at(ui, at);
        }

        left = at.right() + ITEM_INSET;
    }

    let galley = line(ui, text, Weight::Regular, BODY, ink);
    let position = pos2(left, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(position, galley, ink);
    focus_ring(ui, &response, RADIUS);

    // The popup closes on a pointer click by itself, a screen reader's
    // click reaches the item alone.
    if response.clicked() {
        ui.close();
    }

    response
}

pub fn menu_heading(ui: &mut Ui, text: &str) {
    let tokens = tokens::current(ui.ctx());
    let galley = line(ui, text, Weight::SemiBold, SMALL, tokens.secondary);
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), galley.size().y + 10.0),
        Sense::hover(),
    );

    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, text));

    let at = Rect::from_min_size(
        pos2(rect.left() + ITEM_INSET, rect.top() + 6.0),
        galley.size(),
    );

    ui.painter().galley(at.min, galley, tokens.secondary);
}
