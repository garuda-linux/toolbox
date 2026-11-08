//! Encodes key presses and pastes the way xterm does.

#[cfg(test)]
mod tests;

use toolbox_pty::is_plain_text_char;

const ESC: u8 = 0x1b;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// A printable character, already shifted by the keyboard layout.
    Char(char),
    Enter,
    /// Sends DEL (0x7f), Ctrl+Backspace sends BS (0x08).
    Backspace,
    /// Tab (Shift+Tab sends the back tab sequence).
    Tab,
    /// Sends a lone ESC, which TUIs tell from a sequence by timing.
    Escape,
    Up,
    Down,
    Left,
    Right,
    /// Line start, follows the application cursor mode like the arrows.
    Home,
    /// Line end, follows the application cursor mode like the arrows.
    End,
    /// Sends `ESC [ 5 ~`.
    PageUp,
    /// Sends `ESC [ 6 ~`.
    PageDown,
    /// Sends `ESC [ 2 ~`.
    Insert,
    Delete,
    /// F1 to F12. Other numbers send nothing.
    Function(u8),
}

/// The modifier keys held with a key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    /// Ignored for characters, which arrive already shifted.
    pub shift: bool,
    /// Alt (Meta on some keyboards).
    pub alt: bool,
    /// Turns letters and a few symbols into control codes (Ctrl+C is 0x03).
    pub control: bool,
}

#[cfg(test)]
impl Modifiers {
    pub(crate) const ALT: Self = Self {
        shift: false,
        alt: true,
        control: false,
    };
    pub(crate) const CONTROL: Self = Self {
        shift: false,
        alt: false,
        control: true,
    };
    pub(crate) const NONE: Self = Self {
        shift: false,
        alt: false,
        control: false,
    };
    pub(crate) const SHIFT: Self = Self {
        shift: true,
        alt: false,
        control: false,
    };
}

impl Modifiers {
    /// xterm's modifier parameter, `None` without modifiers.
    fn parameter(self) -> Option<u8> {
        let value =
            1_u8 + u8::from(self.shift) + 2_u8 * u8::from(self.alt) + 4_u8 * u8::from(self.control);

        (value > 1_u8).then_some(value)
    }
}

/// The terminal modes that change what keys send. Read it from
/// [`crate::Terminal::key_mode`] right before encoding, since programs
/// switch modes at any time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct KeyMode {
    /// DECCKM: arrows, Home and End send `ESC O` instead of `ESC [`.
    pub application_cursor: bool,
    /// Pasted text goes between `ESC [ 200 ~` and `ESC [ 201 ~`.
    pub bracketed_paste: bool,
}

/// How a key that is not text is spelled.
enum Sequence {
    /// A final letter, `ESC O` in application cursor mode.
    Cursor(u8),
    /// A final letter that is always `ESC O` without modifiers (F1 to F4).
    Keypad(u8),
    Tilde(u8),
}

#[must_use = "the bytes must be written to the program"]
pub fn encode_key(key: Key, modifiers: Modifiers, mode: KeyMode) -> Vec<u8> {
    let alt_prefixed = |bytes: &[u8]| {
        let mut out = Vec::with_capacity(bytes.len().saturating_add(1));
        if modifiers.alt {
            out.push(ESC);
        }

        out.extend_from_slice(bytes);
        out
    };

    match key {
        Key::Char(character) => modifiers
            .control
            .then(|| control_byte(character))
            .flatten()
            .map_or_else(
                || alt_prefixed(character.encode_utf8(&mut [0_u8; 4]).as_bytes()),
                |byte| alt_prefixed(&[byte]),
            ),
        Key::Enter => alt_prefixed(b"\r"),
        Key::Backspace if modifiers.control => alt_prefixed(b"\x08"),
        Key::Backspace => alt_prefixed(b"\x7f"),
        Key::Tab if modifiers.shift => b"\x1b[Z".to_vec(),
        Key::Tab => alt_prefixed(b"\t"),
        Key::Escape => alt_prefixed(b"\x1b"),
        Key::Up => sequence(&Sequence::Cursor(b'A'), modifiers, mode),
        Key::Down => sequence(&Sequence::Cursor(b'B'), modifiers, mode),
        Key::Right => sequence(&Sequence::Cursor(b'C'), modifiers, mode),
        Key::Left => sequence(&Sequence::Cursor(b'D'), modifiers, mode),
        Key::Home => sequence(&Sequence::Cursor(b'H'), modifiers, mode),
        Key::End => sequence(&Sequence::Cursor(b'F'), modifiers, mode),
        Key::Insert => sequence(&Sequence::Tilde(2), modifiers, mode),
        Key::Delete => sequence(&Sequence::Tilde(3), modifiers, mode),
        Key::PageUp => sequence(&Sequence::Tilde(5), modifiers, mode),
        Key::PageDown => sequence(&Sequence::Tilde(6), modifiers, mode),
        Key::Function(number) => function_sequence(number)
            .map(|spelling| sequence(&spelling, modifiers, mode))
            .unwrap_or_default(),
    }
}

/// The bytes the program receives for pasted `text`.
///
/// Control characters other than tab and line breaks are dropped with or
/// without the bracket: a paste is text, so it can neither end the bracket
/// early nor act as Ctrl+C, Ctrl+D or an escape sequence. The clipboard
/// is filled by whatever the user copied from.
#[must_use = "the bytes must be written to the program"]
pub fn encode_paste(text: &str, mode: KeyMode) -> Vec<u8> {
    let inner: String = text
        .chars()
        .filter(|&character| is_plain_text_char(character))
        .collect();

    if mode.bracketed_paste {
        return format!("\x1b[200~{inner}\x1b[201~").into_bytes();
    }

    // Without the bracket the paste must look typed, and Enter sends CR.
    inner.replace("\r\n", "\r").replace('\n', "\r").into_bytes()
}

/// The control code xterm sends for Ctrl+`character`, `None` where
/// Control changes nothing.
fn control_byte(character: char) -> Option<u8> {
    match character {
        'a'..='z' | 'A'..='Z' => u8::try_from(character).ok().map(|byte| byte & 0x1f_u8),
        '@' | ' ' | '2' => Some(0x00_u8),
        '[' | '3' => Some(ESC),
        '\\' | '4' => Some(0x1c_u8),
        ']' | '5' => Some(0x1d_u8),
        '^' | '6' => Some(0x1e_u8),
        '_' | '7' | '/' => Some(0x1f_u8),
        '?' | '8' => Some(0x7f_u8),
        _ => None,
    }
}

const fn function_sequence(number: u8) -> Option<Sequence> {
    Some(match number {
        1 => Sequence::Keypad(b'P'),
        2 => Sequence::Keypad(b'Q'),
        3 => Sequence::Keypad(b'R'),
        4 => Sequence::Keypad(b'S'),
        5 => Sequence::Tilde(15),
        6 => Sequence::Tilde(17),
        7 => Sequence::Tilde(18),
        8 => Sequence::Tilde(19),
        9 => Sequence::Tilde(20),
        10 => Sequence::Tilde(21),
        11 => Sequence::Tilde(23),
        12 => Sequence::Tilde(24),
        _ => return None,
    })
}

fn sequence(spelling: &Sequence, modifiers: Modifiers, mode: KeyMode) -> Vec<u8> {
    let parameter = modifiers.parameter();

    match (spelling, parameter) {
        (Sequence::Cursor(letter), None) if mode.application_cursor => {
            vec![ESC, b'O', *letter]
        }
        (Sequence::Cursor(letter), None) => vec![ESC, b'[', *letter],
        (Sequence::Keypad(letter), None) => vec![ESC, b'O', *letter],
        (Sequence::Cursor(letter) | Sequence::Keypad(letter), Some(value)) => {
            let mut out = format!("\x1b[1;{value}").into_bytes();
            out.push(*letter);
            out
        }
        (Sequence::Tilde(number), None) => format!("\x1b[{number}~").into_bytes(),
        (Sequence::Tilde(number), Some(value)) => format!("\x1b[{number};{value}~").into_bytes(),
    }
}
