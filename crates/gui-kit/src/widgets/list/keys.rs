//! The keys a list's focused row answers (spotifast's song rows and
//! Rerun's list navigation): Up and Down move within the list and stop at
//! its ends, Space flips the row's switch, the Menu key and Shift+F10
//! open the row's menu. Tab is left to egui, so it still reaches the
//! controls inside a row and the next card.

use egui::{FocusDirection, Key, Modifiers, Popup, Ui};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RowKeys {
    pub moved_to: Option<usize>,
    pub toggle: bool,
    pub menu: bool,
}

/// Takes this frame's keys for row `current` of `count`. An open menu
/// keeps them, as it moves among its own items.
pub fn take(ui: &Ui, current: usize, count: usize) -> RowKeys {
    if count == 0 || Popup::is_any_open(ui.ctx()) {
        return RowKeys::default();
    }

    let (down, up, toggle, menu) = ui.input_mut(|input| {
        let down = input.count_and_consume_key(Modifiers::NONE, Key::ArrowDown);
        let up = input.count_and_consume_key(Modifiers::NONE, Key::ArrowUp);
        let toggle = input.consume_key(Modifiers::NONE, Key::Space);
        let shifted = input.consume_key(Modifiers::SHIFT, Key::F10);
        let menu = input.consume_key(Modifiers::NONE, Key::Menu) || shifted;
        (down, up, toggle, menu)
    });

    let moved_to = (down + up > 0)
        .then(|| {
            // egui's own search for a widget in the arrow's direction
            // would land outside the list on the first press.
            ui.memory_mut(|memory| memory.move_focus(FocusDirection::None));
            current
                .saturating_add(down)
                .saturating_sub(up)
                .min(count - 1)
        })
        .filter(|target| *target != current);

    RowKeys {
        moved_to,
        toggle,
        menu,
    }
}
