//! The system clipboard, read for the Console's Ctrl+Shift+V. egui hands
//! clipboard text only to plain Ctrl+V, which the Console passes on to the
//! program, so the app opens the clipboard itself.

use eframe::CreationContext;
use egui_winit::clipboard::Clipboard;
use raw_window_handle::HasDisplayHandle as _;

pub struct SystemClipboard(Clipboard);

impl SystemClipboard {
    /// The clipboard of the window's display. Without a display (tests,
    /// a failed handle) it falls back to what egui-winit offers then.
    #[expect(
        unsafe_code,
        reason = "egui-winit's clipboard needs the raw display handle"
    )]
    pub fn of(cc: &CreationContext<'_>) -> Self {
        let display = cc.display_handle().ok().map(|handle| handle.as_raw());

        // SAFETY: eframe drops the app before its windows, and the display
        // belongs to the event loop, which outlives both.
        Self(unsafe { Clipboard::new(display) })
    }

    pub fn text(&mut self) -> Option<String> {
        self.0.get()
    }
}
