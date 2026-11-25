//! A ⋯ button that opens a menu of actions on a row, and the items of
//! such a menu.

use egui::{
    Align, CornerRadius, Id, Layout, Popup, Rect, Response, Role, Sense, TextStyle, Ui, WidgetInfo,
    pos2, vec2,
};

use super::press::Pointer;
use super::text::cut;
use super::{icon_button, line, rich};
use crate::fonts::{self, BODY, SMALL, Weight};
use crate::frost::Glass;
use crate::icons::{self, Icon};
use crate::theme::{POPOVER_RADIUS, RADIUS};
use crate::tokens::{self, Tokens};
use crate::ui_ext::icon_image;

pub(super) const MENU_WIDTH: f32 = 220.0;
/// Darkly's `MenuItem_MarginWidth`: the room a menu item keeps at its right.
const ITEM_MARGIN: f32 = 5.0;
const ITEM_HEIGHT: f32 = 30.0;
const ITEM_PAD_Y: f32 = 4.0;
/// Darkly's `MenuItem_ItemSpacing` between an icon and the text.
const ICON_GAP: f32 = 4.0;
/// The highlight's inset from the item's rect, and from it while pressed.
const HIGHLIGHT_INSET: f32 = 1.0;
const PRESSED_INSET: f32 = 2.0;
/// From an item's edge to its icon or text.
pub(super) const ITEM_INSET: f32 = 10.0;
/// Between the items of a menu or a list.
pub(super) const LIST_GAP: f32 = 2.0;

/// The widest entry of a menu as the last frame that showed it laid it out,
/// kept between frames under its key.
#[derive(Clone, Copy, Debug, Default)]
struct Widest {
    shown: Option<f32>,
    this_frame: f32,
}

pub fn menu_button(ui: &mut Ui, name: &str, items: impl FnOnce(&mut Ui)) -> Response {
    let button = icon_button(ui, icons::OVERFLOW_MENU, name);
    menu_for(ui, &button, false, items);
    button
}

/// The menu of `button`. A picked entry hands the focus to `button`, as
/// the entry goes with the menu: a dialog the entry opens then has an
/// opener to give the focus back to.
pub(super) fn menu_for(ui: &Ui, button: &Response, open: bool, items: impl FnOnce(&mut Ui)) {
    if open {
        Popup::open_id(ui.ctx(), Popup::default_response_id(button));
    }

    let key = Popup::default_response_id(button);
    let glass = menu_glass(ui);

    // Not justified: an entry stretched to the menu's room would keep the
    // menu as wide as it was, so it could never narrow to its widest entry.
    let picked = Popup::menu(button)
        .width(menu_width(ui, key))
        .layout(Layout::top_down(Align::Min))
        .frame(glass.frame())
        .show(|popup| menu_items(glass, popup, key, items))
        .is_some_and(|shown| shown.inner);

    if picked {
        button.request_focus();
    }
}

/// `items` on `glass`, a list gap apart, with their widths kept under
/// `key`. Returns whether an entry closed the menu.
pub(super) fn menu_items(
    glass: Glass,
    popup: &mut Ui,
    key: Id,
    items: impl FnOnce(&mut Ui),
) -> bool {
    let ctx = popup.ctx().clone();
    let outer = ctx.data(|data| data.get_temp::<Id>(current_menu_id()));
    ctx.data_mut(|data| data.insert_temp(current_menu_id(), key));

    let shown = glass.inside(popup, |list| {
        list.spacing_mut().item_spacing.y = LIST_GAP;
        items(list);
        list.will_parent_close()
    });

    ctx.data_mut(|data| match outer {
        Some(outer) => {
            data.insert_temp(current_menu_id(), outer);
        }
        None => {
            data.remove::<Id>(current_menu_id());
        }
    });

    shown
}

fn widest_id(key: Id) -> Id {
    key.with("widest")
}

fn current_menu_id() -> Id {
    Id::unique("toolbox_gui_kit::menu::current")
}

/// The width of the menu under `key` for this frame: its widest entry
/// of the last frame that showed one, or the fallback before that.
pub(super) fn menu_width(ui: &Ui, key: Id) -> f32 {
    let ctx = ui.ctx();
    let mut widest = ctx
        .data(|data| data.get_temp::<Widest>(widest_id(key)))
        .unwrap_or_default();

    if widest.this_frame > 0.0 {
        widest.shown = Some(widest.this_frame);
    }

    widest.this_frame = 0.0;
    ctx.data_mut(|data| data.insert_temp(widest_id(key), widest));
    let width = widest.shown.unwrap_or(MENU_WIDTH);
    width + tokens::current(ctx).sizes.list_padding.sum().x
}

/// Keeps the width `wide` of an entry for the menu it is in.
fn keep_width(ctx: &egui::Context, wide: f32) {
    let Some(key) = ctx.data(|data| data.get_temp::<Id>(current_menu_id())) else {
        return;
    };

    ctx.data_mut(|data| {
        let mut widest = data.get_temp::<Widest>(widest_id(key)).unwrap_or_default();

        widest.this_frame = widest.this_frame.max(wide);
        data.insert_temp(widest_id(key), widest);
    });
}

/// The widest entry the menu showed last frame. Entries fill the menu's
/// room, which its area keeps, so without this the menu never narrows.
/// Unknown in the first frame, where the room decides.
fn widest_shown(ctx: &egui::Context) -> f32 {
    let Some(key) = ctx.data(|data| data.get_temp::<Id>(current_menu_id())) else {
        return f32::INFINITY;
    };

    ctx.data(|data| data.get_temp::<Widest>(widest_id(key)))
        .and_then(|widest| widest.shown)
        .unwrap_or(f32::INFINITY)
}

/// The width a line of a menu takes: the room, or the widest entry when the
/// menu is narrower than its room.
fn menu_room(ui: &Ui) -> f32 {
    ui.available_width().min(widest_shown(ui.ctx()))
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
    let width = menu_room(ui);
    let height = item_height(ui);
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::MenuItem, ui.is_enabled(), text));

    // Darkly's menus show the keyboard's entry as the pointer does, so a
    // key-focused item takes the hover fill and no ring.
    let pointer = Pointer::of_keyboard(ui, &response);
    let filled = ui.is_enabled() && pointer != Pointer::Rest;
    if filled {
        paint_item_fill(ui, rect, pointer, &tokens);
    }

    // A disabled ui fades it once, see `press.rs`. On the fill Darkly's
    // strong focus takes the highlighted text colour.
    let ink = if filled {
        tokens.highlighted_text
    } else {
        tokens.text
    };

    let mut left = rect.left() + ITEM_INSET;
    if room {
        let side = ui.text_style_height(&TextStyle::Body);
        let at = Rect::from_min_size(pos2(left, rect.center().y - side / 2.0), vec2(side, side));
        if let Some(icon) = icon {
            icon_image(ui, icon, ink).paint_at(ui, at);
        }

        left = at.right() + ICON_GAP;
    }

    let whole = line(ui, text, Weight::Regular, BODY, ink).size().x;
    keep_width(ui.ctx(), left - rect.left() + whole + ITEM_MARGIN);

    let label_room = (rect.right() - ITEM_MARGIN - left).max(0.0);
    let galley = cut(ui, rich(ui, text, Weight::Regular, BODY, ink), label_room);
    let was_cut = galley.elided;
    let position = pos2(left, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(position, galley, ink);

    // The popup closes on a pointer click by itself, a screen reader's
    // click reaches the item alone.
    if response.clicked() {
        ui.close();
    }

    if was_cut {
        response.on_hover_text(text)
    } else {
        response
    }
}

/// The height of a Plasma QML list entry (`ItemDelegate` in System Settings),
/// taller when a large font needs it. Darkly's own item rows read cramped
/// beside 34 point buttons.
pub(super) fn item_height(ui: &Ui) -> f32 {
    let font = fonts::font(ui.style(), Weight::Regular, BODY);
    let line = ui.ctx().fonts_mut(|fonts| fonts.row_height(&font));
    ITEM_PAD_Y.mul_add(2.0, line).max(ITEM_HEIGHT)
}

/// The fill of a menu item under the pointer or the keyboard, one point
/// inside its rect, and two while pressed. Darkly fills it solid, the hover
/// colour on the view and the focus colour while pressed.
fn paint_item_fill(ui: &Ui, rect: Rect, pointer: Pointer, tokens: &Tokens) {
    let (fill, inset) = match pointer {
        Pointer::Rest => return,
        Pointer::Hovered => (tokens.decoration_hover, HIGHLIGHT_INSET),
        Pointer::Pressed => (tokens.decoration_focus, PRESSED_INSET),
    };

    ui.painter()
        .rect_filled(rect.shrink(inset), CornerRadius::same(RADIUS), fill);
}

/// A 1 point rule between the groups of a menu, inset from both edges by
/// the item inset, in the separator colour.
pub fn menu_separator(ui: &mut Ui) {
    let tokens = tokens::current(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(menu_room(ui), 1.0), Sense::hover());
    let y = rect.top() + 0.5;
    let stroke = egui::Stroke::new(1.0, tokens.separator);
    ui.painter().line_segment(
        [
            pos2(rect.left() + ITEM_INSET, y),
            pos2(rect.right() - ITEM_INSET, y),
        ],
        stroke,
    );
}

pub fn menu_heading(ui: &mut Ui, text: &str) {
    let tokens = tokens::current(ui.ctx());
    let galley = line(ui, text, Weight::SemiBold, SMALL, tokens.secondary);
    let (rect, response) =
        ui.allocate_exact_size(vec2(menu_room(ui), galley.size().y + 10.0), Sense::hover());

    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, text));

    let at = Rect::from_min_size(
        pos2(rect.left() + ITEM_INSET, rect.top() + 6.0),
        galley.size(),
    );

    ui.painter().galley(at.min, galley, tokens.secondary);
}
