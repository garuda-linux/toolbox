//! The ⋯ button at a row's right, which holds every action behind the
//! row's one visible button.
//!
//! A row shows the action most people want, Run now before Queue, and
//! keeps the rest here, so a card reads as a list of things rather than
//! a wall of buttons. A Danger action that asks first stays in the menu
//! and still asks.

use egui::{Key, Popup, Response, Ui};
use toolbox_gui_kit::widgets::{menu_button, menu_item};

use crate::i18n;

pub fn more(ui: &mut Ui, row: &str, items: impl FnOnce(&mut Ui)) -> Response {
    let alt = i18n::text_with(ui.ctx(), "row-more", &[("name", row)]);
    let button = menu_button(ui, &alt, items);
    return_focus(ui, &button);
    button
}

/// Gives the focus back to `button` once a key closed its menu, as
/// Escape or an item picked with Enter leaves nothing focused and Tab
/// would start over at the window's first control. A pointer close gets
/// no ring, and focus another control took stays.
fn return_focus(ui: &Ui, button: &Response) {
    let ctx = ui.ctx();
    let open = Popup::is_id_open(ctx, Popup::default_response_id(button));
    let was_id = button.id.with("more_was_open");
    let was_open = ctx
        .data(|data| data.get_temp::<bool>(was_id))
        .unwrap_or(false);

    if open != was_open {
        ctx.data_mut(|data| data.insert_temp(was_id, open));
    }

    let nothing_focused = ctx.memory(|memory| memory.focused().is_none());
    let by_key = ctx.input(|input| {
        [Key::Escape, Key::Enter, Key::Space]
            .into_iter()
            .any(|key| input.key_pressed(key))
    });

    if was_open && !open {
        // The Escape that closed the menu is spent: the Console, drawn
        // later in the frame, would close on it too.
        ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Escape));

        if nothing_focused && by_key {
            button.request_focus();
        }
    }
}

/// One action of a ⋯ menu, labelled by the text `id` and resting while
/// not `enabled`. Returns whether it was clicked.
pub fn item(ui: &mut Ui, id: &str, enabled: bool) -> bool {
    let text = i18n::text(ui.ctx(), id);
    text_item(ui, &text, enabled)
}

pub fn text_item(ui: &mut Ui, text: &str, enabled: bool) -> bool {
    ui.add_enabled_ui(enabled, |ui| menu_item(ui, text))
        .inner
        .clicked()
}

#[cfg(test)]
mod tests;
