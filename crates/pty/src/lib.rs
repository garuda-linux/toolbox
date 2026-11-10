//! Programs in a pseudo-terminal, and their output made fit for a GUI
//! console. Linked by the root helper and the app, so it stays free of an
//! async runtime.
//!
//! A caller spawns a [`Session`], reads its [`Chunk`]s and types into it.
//! Cancelling and waiting are polled, so the only threads are the output
//! reader and the input writer.

mod ansi;
mod cancel;
mod command;
mod reader;
mod runs;
mod session;
mod size;
mod spawn;
mod style;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
mod writer;

pub use ansi::{is_plain_text_char, strip_ansi};
pub use cancel::Signal;
pub use command::PtyCommand;
pub use reader::Chunk;
pub use runs::styled_lines;
pub use session::{Exit, Session};
pub use size::{DEFAULT_SIZE, TerminalSize, clamped};
pub use style::{Color, Run, Style};
