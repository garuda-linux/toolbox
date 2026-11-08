//! What the engine hands back while it parses: replies the program waits
//! for. The engine's listener takes `&self`, so the outbox is shared
//! behind a lock. A mutex rather than a `RefCell` keeps
//! [`crate::Terminal`] `Send` for the app's tokio tasks.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use alacritty_terminal::event::{Event, EventListener, WindowSize};
use parking_lot::Mutex;

#[derive(Clone, Debug, Default)]
pub struct Outbox {
    replies: Arc<Mutex<Vec<u8>>>,
}

impl Outbox {
    pub fn take_replies(&self) -> Vec<u8> {
        std::mem::take(&mut *self.replies.lock())
    }
}

impl EventListener for Outbox {
    fn send_event(&self, event: Event) {
        let mut replies = self.replies.lock();

        match event {
            Event::PtyWrite(text) => replies.extend_from_slice(text.as_bytes()),
            // The pixel size is unknown here. Zero is xterm's answer for
            // "not available", and the program does not hang waiting.
            Event::TextAreaSizeRequest(format) => {
                let unknown = WindowSize {
                    num_lines: 0,
                    num_cols: 0,
                    cell_width: 0,
                    cell_height: 0,
                };

                replies.extend_from_slice(format(unknown).as_bytes());
            }
            // Titles and bells go nowhere: the Console shows no title
            // and sounds no bell. Colour queries stay unanswered and
            // programs get no clipboard, the rest serves no window.
            Event::Title(_)
            | Event::ResetTitle
            | Event::Bell
            | Event::ColorRequest(..)
            | Event::ClipboardStore(..)
            | Event::ClipboardLoad(..)
            | Event::MouseCursorDirty
            | Event::CursorBlinkingChange
            | Event::Wakeup
            | Event::Exit
            | Event::ChildExit(_) => {}
        }
    }
}
