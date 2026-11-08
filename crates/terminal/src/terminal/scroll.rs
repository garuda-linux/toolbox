//! The view's own scroll position, kept by the engine as alacritty keeps
//! it: output that arrives while the view is scrolled up moves the offset
//! along, so the lines the user reads stay where they are.

#[cfg(test)]
mod tests;

use alacritty_terminal::grid::Scroll;

use crate::Terminal;
use crate::screen::Screen;

impl Terminal {
    /// Scrolls the view `lines` up (positive) or down (negative), within
    /// the scrollback.
    pub fn scroll(&mut self, lines: i32) {
        self.term.scroll_display(Scroll::Delta(lines));
    }

    pub fn scroll_to_bottom(&mut self) {
        self.term.scroll_display(Scroll::Bottom);
    }

    /// How many lines the view is scrolled up. 0 is the live screen.
    pub fn display_offset(&self) -> usize {
        self.term.grid().display_offset()
    }

    /// The screen as the view shows it: scrolled by
    /// [`Self::display_offset`], the cursor moved down with it and hidden
    /// once it is below the view.
    pub fn view(&self) -> Screen {
        let offset = self.display_offset();
        let lines = self.screen_lines();
        let mut cursor = self.cursor();
        cursor.row = cursor.row.saturating_add(offset);
        cursor.visible &= cursor.row < lines;

        Screen {
            rows: self.rows(self.history_len().saturating_sub(offset), lines),
            cursor,
        }
    }
}
