//! A row's menu, after Rerun's context menus: the ⋯ button at the row's
//! right, and the same items on a right click anywhere on the row or the
//! Menu key on the focused row.

use egui::{Align, Layout, Popup, PopupAnchor, PopupKind, Pos2, Response, Ui, vec2};

use super::menu::{MENU_WIDTH, menu_for, menu_glass, menu_items};
use super::{RowState, icon_button};
use crate::icons;

/// The ⋯ button `alt` names, with `items` in its menu and on a right
/// click of the row.
///
/// The Menu key or Shift+F10 on the focused row opens it and moves the
/// focus to the button, so Down enters the menu. In a disabled `ui`
/// neither opens.
pub fn row_menu(
    ui: &mut Ui,
    row: &RowState,
    alt: &str,
    mut items: impl FnMut(&mut Ui),
) -> Response {
    let button = icon_button(ui, icons::OVERFLOW_MENU, alt);
    let enabled = ui.is_enabled();
    let open = enabled && row.menu;
    if open {
        button.request_focus();
    }

    menu_for(ui, &button, open, &mut items);
    if enabled {
        context_menu(ui, row, alt, None, &mut items);
    }

    button
}

/// `items` on a right click of a row without a ⋯ button, or on the Menu
/// key, then below the row's start. `name` names the menu.
pub fn row_context_menu(ui: &Ui, row: &RowState, name: &str, mut items: impl FnMut(&mut Ui)) {
    let padding = ui.spacing().item_spacing.x;
    let below = row
        .menu
        .then(|| row.response.rect.left_bottom() + vec2(padding, 0.0));

    context_menu(ui, row, name, below, &mut items);
}

fn context_menu(
    ui: &Ui,
    row: &RowState,
    name: &str,
    keyboard_at: Option<Pos2>,
    items: &mut dyn FnMut(&mut Ui),
) {
    let ctx = ui.ctx();
    let id = row.response.id.with("context-menu");
    let at_id = id.with("at");
    let opened_at = if row.response.secondary_clicked() {
        ctx.pointer_interact_pos()
    } else {
        keyboard_at
    };

    if let Some(at) = opened_at {
        ctx.data_mut(|data| data.insert_temp(at_id, at));
        Popup::open_id(ctx, id);
    }

    let Some(at) = ctx.data(|data| data.get_temp::<Pos2>(at_id)) else {
        return;
    };

    if !Popup::is_id_open(ctx, id) {
        return;
    }

    let glass = menu_glass(ui);
    Popup::new(
        id,
        ctx.clone(),
        PopupAnchor::Position(at),
        row.response.layer_id,
    )
    .kind(PopupKind::Menu)
    .layout(Layout::top_down_justified(Align::Min))
    .open_memory(None)
    .anchor_widget(row.response.id)
    .accessible_name(name)
    .width(MENU_WIDTH)
    .frame(glass.frame())
    .show(|popup| {
        menu_items(glass, popup, items);
    });
}
