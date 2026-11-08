//! The Console's terminal emulator, free of any UI toolkit and runtime.
//!
//! Programs run in a pseudo-terminal, as the user through `toolbox-pty` or
//! as root inside the helper. Their raw output goes into
//! [`Terminal::feed`]. The UI draws [`Terminal::view`] (rows of styled runs
//! plus the cursor) with theme colours each frame, and sends what
//! [`encode_key`], [`encode_paste`] and [`Terminal::take_replies`] return
//! back to the program. The engine is `alacritty_terminal`.

mod color;
mod keys;
mod mouse;
mod outbox;
mod screen;
mod terminal;
#[cfg(test)]
mod testing;

pub use keys::{Key, KeyMode, Modifiers, encode_key, encode_paste};
pub use mouse::{
    MouseAction, MouseButton, MouseEncoding, MouseEvent, MouseMode, MouseReporting,
    encode_alternate_scroll, encode_mouse,
};
pub use screen::{Cursor, CursorShape, Row, Screen};
pub use terminal::{CellPoint, Terminal, TerminalSize};
pub use toolbox_pty::{Color, Run, Style};
