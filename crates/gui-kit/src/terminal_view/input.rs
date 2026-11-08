#[cfg(test)]
mod tests;

use egui::{Event, Key as EguiKey, Modifiers};
use toolbox_terminal::{Key, KeyMode, Modifiers as KeyModifiers, encode_key};

/// What egui's clipboard shortcuts become: the control codes they are in
/// any other terminal.
const INTERRUPT: u8 = 0x03;
const CUT: u8 = 0x18;
const LITERAL_NEXT: u8 = 0x16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Bytes(Vec<u8>),
    /// Ctrl+Shift+C: copy the selection.
    Copy,
    /// Ctrl+Shift+V: paste the clipboard, which the app reads.
    Paste,
    /// Ctrl+Tab and Ctrl+Shift+Tab: move focus out of the Console.
    Leave {
        backward: bool,
    },
    Nothing,
}

/// `event` with the modifiers `held` this frame.
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "every other egui event means nothing to the program"
)]
pub fn translate(event: &Event, held: Modifiers, mode: KeyMode) -> Input {
    match event {
        // With Ctrl or Alt the key event carries it, egui may send the
        // text as well.
        Event::Text(_) if held.ctrl || held.alt => Input::Nothing,
        Event::Text(text) => Input::Bytes(text.as_bytes().to_vec()),
        // egui-winit turns Ctrl+C, X and V into clipboard events. In a
        // terminal they belong to the program.
        Event::Copy => Input::Bytes(vec![INTERRUPT]),
        Event::Cut => Input::Bytes(vec![CUT]),
        Event::Paste(_) => Input::Bytes(vec![LITERAL_NEXT]),
        Event::Key {
            key,
            physical_key: _,
            pressed: true,
            repeat: _,
            modifiers,
        } => key_input(*key, *modifiers, mode),
        _ => Input::Nothing,
    }
}

fn key_input(key: EguiKey, modifiers: Modifiers, mode: KeyMode) -> Input {
    match (key, modifiers.ctrl, modifiers.shift) {
        (EguiKey::Tab, true, backward) => return Input::Leave { backward },
        (EguiKey::C, true, true) => return Input::Copy,
        (EguiKey::V, true, true) => return Input::Paste,
        _ => {}
    }

    let held = key_modifiers(modifiers);
    if let Some(special) = special_key(key) {
        return Input::Bytes(encode_key(special, held, mode));
    }

    // Letters and digits alone arrive as text. Only Ctrl and Alt make
    // them keys of their own.
    match character(key) {
        Some(character) if modifiers.ctrl || modifiers.alt => {
            Input::Bytes(encode_key(Key::Char(character), held, mode))
        }
        Some(_) | None => Input::Nothing,
    }
}

pub(super) const fn key_modifiers(modifiers: Modifiers) -> KeyModifiers {
    KeyModifiers {
        shift: modifiers.shift,
        alt: modifiers.alt,
        control: modifiers.ctrl,
    }
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "egui knows many keys a terminal has no sequence for"
)]
const fn special_key(key: EguiKey) -> Option<Key> {
    let key = match key {
        EguiKey::Enter => Key::Enter,
        EguiKey::Backspace => Key::Backspace,
        EguiKey::Tab => Key::Tab,
        EguiKey::Escape => Key::Escape,
        EguiKey::ArrowUp => Key::Up,
        EguiKey::ArrowDown => Key::Down,
        EguiKey::ArrowLeft => Key::Left,
        EguiKey::ArrowRight => Key::Right,
        EguiKey::Home => Key::Home,
        EguiKey::End => Key::End,
        EguiKey::PageUp => Key::PageUp,
        EguiKey::PageDown => Key::PageDown,
        EguiKey::Insert => Key::Insert,
        EguiKey::Delete => Key::Delete,
        EguiKey::F1 => Key::Function(1),
        EguiKey::F2 => Key::Function(2),
        EguiKey::F3 => Key::Function(3),
        EguiKey::F4 => Key::Function(4),
        EguiKey::F5 => Key::Function(5),
        EguiKey::F6 => Key::Function(6),
        EguiKey::F7 => Key::Function(7),
        EguiKey::F8 => Key::Function(8),
        EguiKey::F9 => Key::Function(9),
        EguiKey::F10 => Key::Function(10),
        EguiKey::F11 => Key::Function(11),
        EguiKey::F12 => Key::Function(12),
        _ => return None,
    };

    Some(key)
}

fn character(key: EguiKey) -> Option<char> {
    if key == EguiKey::Space {
        return Some(' ');
    }

    let mut name = key.name().chars();
    let first = name.next()?;

    (name.next().is_none() && first.is_ascii_alphanumeric()).then(|| first.to_ascii_lowercase())
}
