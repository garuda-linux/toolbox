//! What the mouse sends to a program that asked for it (xterm's mouse
//! reporting, as alacritty encodes it), and the wheel's arrow keys in
//! full-screen programs that only read the keyboard.

#[cfg(test)]
mod tests;

use crate::Modifiers;

// xterm's button codes and the bits added to them.
const SHIFT: u16 = 4;
const ALT: u16 = 8;
const CONTROL: u16 = 16;
const MOTION: u16 = 32;
const WHEEL_UP: u16 = 64;
const NO_BUTTON: u16 = 3;
/// The classic form sends each number plus 32, so it is a printable byte.
const CLASSIC_OFFSET: u16 = 32;
/// The largest cell value plus offset xterm's UTF-8 form spells, two bytes.
const UTF8_LIMIT: u32 = 2047;

/// A mouse button, the wheel counted as two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseAction {
    Press,
    Release,
    Motion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MouseEvent {
    pub button: Option<MouseButton>,
    pub action: MouseAction,
    /// Cell column from the left, 0 based.
    pub column: usize,
    /// Screen row from the top, 0 based.
    pub row: usize,
    pub modifiers: Modifiers,
}

/// What the program wants to hear about. Each level includes the one
/// before it, as xterm's modes 1000, 1002 and 1003 do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MouseReporting {
    #[default]
    Off,
    Clicks,
    /// Also motion while a button is held.
    Drag,
    /// Also any motion.
    Motion,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MouseEncoding {
    /// `ESC [ M` and three bytes, cells up to 223.
    #[default]
    Classic,
    /// The classic form with cells beyond its limit as UTF-8.
    Utf8,
    /// `ESC [ <`, without a size limit.
    Sgr,
}

/// The mouse modes the program set. Read it from
/// [`crate::Terminal::mouse_mode`] right before encoding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MouseMode {
    pub reporting: MouseReporting,
    pub encoding: MouseEncoding,
    /// The wheel sends arrow keys, on the alternate screen only.
    pub alternate_scroll: bool,
}

/// The bytes for `event`, `None` when the program did not ask for it or
/// the classic form cannot hold its cell.
#[must_use = "the bytes must be written to the program"]
pub fn encode_mouse(event: MouseEvent, mode: MouseMode) -> Option<Vec<u8>> {
    if !wanted(event, mode) {
        return None;
    }

    let modifiers = u16::from(event.modifiers.shift) * SHIFT
        + u16::from(event.modifiers.alt) * ALT
        + u16::from(event.modifiers.control) * CONTROL;

    let button = match event.button {
        Some(MouseButton::Left) => 0,
        Some(MouseButton::Middle) => 1,
        Some(MouseButton::Right) => 2,
        Some(MouseButton::WheelUp) => WHEEL_UP,
        Some(MouseButton::WheelDown) => WHEEL_UP + 1,
        None => NO_BUTTON,
    };

    let motion = if event.action == MouseAction::Motion {
        MOTION
    } else {
        0
    };

    if mode.encoding == MouseEncoding::Sgr {
        let end = if event.action == MouseAction::Release {
            'm'
        } else {
            'M'
        };

        let code = button + motion + modifiers;
        let (column, row) = (event.column.saturating_add(1), event.row.saturating_add(1));
        return Some(format!("\x1b[<{code};{column};{row}{end}").into_bytes());
    }

    // The classic form has no button for a release, only "released".
    let code = if event.action == MouseAction::Release {
        NO_BUTTON + modifiers
    } else {
        button + motion + modifiers
    };

    let mut bytes = b"\x1b[M".to_vec();
    bytes.push(u8::try_from(CLASSIC_OFFSET + code).ok()?);
    let utf8 = mode.encoding == MouseEncoding::Utf8;
    push_position(&mut bytes, event.column, utf8)?;
    push_position(&mut bytes, event.row, utf8)?;
    Some(bytes)
}

/// Arrow keys for `lines` of wheel, up for positive.
#[must_use = "the bytes must be written to the program"]
pub fn encode_alternate_scroll(lines: i32) -> Vec<u8> {
    let arrow: &[u8] = if lines > 0 { b"\x1bOA" } else { b"\x1bOB" };
    let count = usize::try_from(lines.unsigned_abs()).unwrap_or_default();

    arrow.repeat(count)
}

/// Whether the program's modes ask for `event`. The wheel has no
/// release.
const fn wanted(event: MouseEvent, mode: MouseMode) -> bool {
    let level = mode.reporting;
    let wheel = matches!(
        event.button,
        Some(MouseButton::WheelUp | MouseButton::WheelDown)
    );

    match (event.action, event.button) {
        (MouseAction::Release, _) if wheel => false,
        (MouseAction::Press | MouseAction::Release, Some(_)) => {
            !matches!(level, MouseReporting::Off)
        }
        (MouseAction::Motion, Some(_)) => {
            matches!(level, MouseReporting::Drag | MouseReporting::Motion)
        }
        (MouseAction::Motion, None) => matches!(level, MouseReporting::Motion),
        (MouseAction::Press | MouseAction::Release, None) => false,
    }
}

/// A 0 based cell as the classic form spells it: 32 plus the 1 based
/// position, one byte, or one UTF-8 character in UTF-8 mode.
fn push_position(bytes: &mut Vec<u8>, position: usize, utf8: bool) -> Option<()> {
    let value = u32::try_from(position)
        .ok()?
        .checked_add(u32::from(CLASSIC_OFFSET) + 1)?;

    if utf8 {
        let character = char::from_u32(value).filter(|_| value <= UTF8_LIMIT)?;
        let mut buffer = [0_u8; 4];
        bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
    } else {
        bytes.push(u8::try_from(value).ok()?);
    }

    Some(())
}
