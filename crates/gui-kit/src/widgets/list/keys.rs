//! The keys a list's focused row answers beyond its moves (spotifast's
//! song rows and Rerun's list navigation): Space flips the row's switch,
//! the Menu key and Shift+F10 open the row's menu. Up, Down, Home and End
//! move the list's Tab stop, see [`super::super::roving`]. Tab is left to
//! egui, so it still reaches the controls inside a row and the next card.

use egui::{Key, Modifiers, Popup, Ui};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RowKeys {
    pub toggle: bool,
    pub menu: bool,
}

/// Takes this frame's keys for the focused row `held`, if the focus is on
/// one. An open menu keeps them, as it moves among its own items.
pub fn take(ui: &Ui, held: Option<usize>) -> RowKeys {
    if held.is_none() || Popup::is_any_open(ui.ctx()) {
        return RowKeys::default();
    }

    let (toggle, menu) = ui.input_mut(|input| {
        let toggle = input.consume_key(Modifiers::NONE, Key::Space);
        let shifted = input.consume_key(Modifiers::SHIFT, Key::F10);
        let menu = input.consume_key(Modifiers::NONE, Key::Menu) || shifted;
        (toggle, menu)
    });

    RowKeys { toggle, menu }
}
