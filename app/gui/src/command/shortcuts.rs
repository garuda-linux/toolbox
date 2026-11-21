use std::cmp::Reverse;
use std::sync::LazyLock;

use egui::{InputState, Key, KeyboardShortcut, Modifiers};

use super::UiCommand;
use crate::page::PageId;

/// Ctrl and a digit open the first nine pages, in sidebar order.
const PAGE_KEYS: [Key; 9] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

/// The app's shortcuts never change, so the frame does not list and rank
/// them again.
static BOUND: LazyLock<Vec<(KeyboardShortcut, UiCommand)>> = LazyLock::new(|| {
    let pages = PageId::ALL.into_iter().map(UiCommand::Navigate);

    // `+` needs Shift on a US keyboard, so browsers zoom in on Ctrl+= too.
    let zoom_in = KeyboardShortcut::new(Modifiers::COMMAND, Key::Equals);
    let bound = UiCommand::ACTIONS
        .into_iter()
        .chain(pages)
        .filter_map(|command| Some((command.shortcut()?, command)))
        .chain([(zoom_in, UiCommand::ZoomIn)])
        .collect();

    ranked(bound)
});

impl UiCommand {
    pub fn shortcut(self) -> Option<KeyboardShortcut> {
        let key = match self {
            Self::Navigate(page) => PageId::ALL
                .iter()
                .zip(PAGE_KEYS)
                .find_map(|(each, key)| (*each == page).then_some(key))?,
            Self::Apply => Key::Enter,
            Self::ToggleSidebar => Key::B,
            Self::ToggleConsole => Key::J,
            Self::OpenPalette => Key::K,
            Self::FocusSearch => Key::F,
            Self::ZoomIn => Key::Plus,
            Self::ZoomOut => Key::Minus,
            Self::ZoomReset => Key::Num0,
            Self::Help => return Some(KeyboardShortcut::new(Modifiers::NONE, Key::F1)),
            Self::ReviewChanges | Self::ClearChanges => return None,
            Self::Quit => Key::Q,
        };

        Some(KeyboardShortcut::new(Modifiers::COMMAND, key))
    }

    /// Takes every shortcut pressed this frame out of `input`, so no
    /// widget sees it too. While a text field has the focus, the ones that
    /// edit text there leave it to the field.
    pub fn pressed(input: &mut InputState, text_focused: bool) -> Vec<Self> {
        consume(input, text_focused, &BOUND)
    }
}

/// egui counts Ctrl+Shift+S as Ctrl+S pressed too, as Shift and
/// Alt sometimes only make a key, so the shortcut with the most modifiers
/// comes first and takes the press.
pub fn ranked<C>(mut bound: Vec<(KeyboardShortcut, C)>) -> Vec<(KeyboardShortcut, C)> {
    bound.sort_by_key(|(shortcut, _)| Reverse(modifier_count(shortcut.modifiers)));
    bound
}

/// The commands of the shortcuts pressed, tried in the order of `bound`,
/// which [`ranked`] makes.
pub fn consume<C>(
    input: &mut InputState,
    text_focused: bool,
    bound: &[(KeyboardShortcut, C)],
) -> Vec<C>
where
    C: Copy,
{
    bound
        .iter()
        .filter(|(shortcut, _)| !(text_focused && edits_text(*shortcut)))
        .filter(|(shortcut, _)| input.consume_shortcut(shortcut))
        .map(|(_, command)| *command)
        .collect()
}

/// egui's keys of a text field, and undo and redo, which a field has too.
fn edits_text(shortcut: KeyboardShortcut) -> bool {
    let modifiers = shortcut.modifiers;
    shortcut.conflicts_with_text_editing()
        || ((modifiers.command || modifiers.ctrl || modifiers.mac_cmd)
            && matches!(shortcut.logical_key, Key::Z | Key::Y))
}

/// Ctrl and the command key are one modifier on Linux, which egui sets
/// both of.
fn modifier_count(modifiers: Modifiers) -> u8 {
    [
        modifiers.alt,
        modifiers.shift,
        modifiers.ctrl || modifiers.command || modifiers.mac_cmd,
    ]
    .into_iter()
    .map(u8::from)
    .sum()
}
