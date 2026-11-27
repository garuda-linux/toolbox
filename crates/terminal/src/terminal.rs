//! The terminal: program output in, a drawable screen out.

#[cfg(test)]
mod tests;

mod osc;
mod scroll;
mod select;

use std::time::Duration;

use alacritty_terminal::Term;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Osc52, TermMode};
use alacritty_terminal::vte::ansi::{Processor, Timeout};
/// The pty's size type: the program and its screen share one.
pub use toolbox_pty::TerminalSize;

use crate::outbox::Outbox;
use crate::screen::{self, Cursor, Row};
use crate::{KeyMode, MouseEncoding, MouseMode, MouseReporting};

/// Lines kept above the screen.
pub const SCROLLBACK_LINES: usize = 5000;

/// A cell position counted over scrollback and screen together: line 0 is
/// the oldest scrollback line, line [`Terminal::history_len`] the top
/// screen row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellPoint {
    /// Line from the top of the scrollback.
    pub line: usize,
    /// Cell column from the left.
    pub column: usize,
}

impl CellPoint {
    pub fn in_order(self, other: Self) -> (Self, Self) {
        if self <= other {
            (self, other)
        } else {
            (other, self)
        }
    }
}

/// The engine's size type. A local struct because the orphan rule forbids
/// implementing its trait on [`TerminalSize`].
struct Dims {
    columns: usize,
    lines: usize,
}

impl Dims {
    /// The engine needs at least two columns and one line.
    fn of(size: TerminalSize) -> Self {
        Self {
            columns: usize::from(size.columns.max(2)),
            lines: usize::from(size.rows.max(1)),
        }
    }
}

impl Dimensions for Dims {
    fn total_lines(&self) -> usize {
        self.lines
    }

    fn screen_lines(&self) -> usize {
        self.lines
    }

    fn columns(&self) -> usize {
        self.columns
    }
}

/// Applies mode 2026 updates at once: the terminal owns no timer and the UI
/// redraws on its own frame clock.
#[derive(Default)]
struct Unsynchronized;

impl Timeout for Unsynchronized {
    fn set_timeout(&mut self, _duration: Duration) {}

    fn clear_timeout(&mut self) {}

    fn pending_timeout(&self) -> bool {
        false
    }
}

/// A VT terminal emulator without a window.
pub struct Terminal {
    term: Term<Outbox>,
    parser: Processor<Unsynchronized>,
    osc: osc::Tracker,
    outbox: Outbox,
    size: TerminalSize,
}

impl Terminal {
    /// An empty terminal of `size` with [`SCROLLBACK_LINES`] of history.
    #[must_use = "a terminal does nothing until fed"]
    pub fn new(size: TerminalSize) -> Self {
        let outbox = Outbox::default();
        let config = Config {
            scrolling_history: SCROLLBACK_LINES,
            osc52: Osc52::Disabled,
            ..Config::default()
        };

        let term = Term::new(config, &Dims::of(size), outbox.clone());

        Self {
            term,
            parser: Processor::new(),
            osc: osc::Tracker::default(),
            outbox,
            size,
        }
    }

    /// Parses output of the program. Split escape sequences and UTF-8
    /// characters across calls are fine. An operating system command string
    /// past its limit is cancelled, as the parser keeps it otherwise without
    /// end.
    pub fn feed(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }

        let mut kept_from = 0;
        for (index, &byte) in bytes.iter().enumerate() {
            if self.osc.step(byte) == osc::Step::Overlong {
                let kept = bytes.get(kept_from..=index).unwrap_or_default();
                self.parser.advance(&mut self.term, kept);
                self.parser.advance(&mut self.term, &[osc::CAN]);
                kept_from = index + 1;
            }
        }

        let rest = bytes.get(kept_from..).unwrap_or_default();
        self.parser.advance(&mut self.term, rest);
    }

    /// [`Self::feed`] for output that arrived as text, such as the
    /// helper's `Event::Output`.
    pub fn feed_str(&mut self, text: &str) {
        self.feed(text.as_bytes());
    }

    /// The program must be told too, with the pty's resize or `Answer::Resize`.
    pub fn resize(&mut self, size: TerminalSize) {
        if size == self.size {
            return;
        }

        self.size = size;
        self.term.resize(Dims::of(size));
    }

    pub const fn size(&self) -> TerminalSize {
        self.size
    }

    /// The modes [`crate::encode_key`] and [`crate::encode_paste`] need.
    pub fn key_mode(&self) -> KeyMode {
        let mode = self.term.mode();

        KeyMode {
            application_cursor: mode.contains(TermMode::APP_CURSOR),
            bracketed_paste: mode.contains(TermMode::BRACKETED_PASTE),
        }
    }

    /// The modes [`crate::encode_mouse`] needs, and whether the wheel
    /// sends arrow keys.
    pub fn mouse_mode(&self) -> MouseMode {
        let mode = self.term.mode();

        let reporting = if mode.contains(TermMode::MOUSE_MOTION) {
            MouseReporting::Motion
        } else if mode.contains(TermMode::MOUSE_DRAG) {
            MouseReporting::Drag
        } else if mode.contains(TermMode::MOUSE_REPORT_CLICK) {
            MouseReporting::Clicks
        } else {
            MouseReporting::Off
        };

        let encoding = if mode.contains(TermMode::SGR_MOUSE) {
            MouseEncoding::Sgr
        } else if mode.contains(TermMode::UTF8_MOUSE) {
            MouseEncoding::Utf8
        } else {
            MouseEncoding::Classic
        };

        MouseMode {
            reporting,
            encoding,
            alternate_scroll: mode.contains(TermMode::ALTERNATE_SCROLL)
                && mode.contains(TermMode::ALT_SCREEN),
        }
    }

    /// Bytes the program asked for (device status and attribute reports),
    /// emptied. Write them to the program as they are.
    #[must_use = "the replies must be written to the program"]
    pub fn take_replies(&mut self) -> Vec<u8> {
        self.outbox.take_replies()
    }

    /// How many lines scrolled off the top are kept. Zero on the
    /// alternate screen, which has no scrollback.
    pub fn history_len(&self) -> usize {
        self.term.grid().history_size()
    }

    /// Scrollback and screen as text, one line per line the program wrote
    /// (wrapped lines joined), trailing blanks dropped.
    pub fn to_plain_text(&self) -> String {
        let last = CellPoint {
            line: self.total_lines().saturating_sub(1),
            column: self.columns().saturating_sub(1),
        };

        self.selection_text(CellPoint::default(), last)
            .trim_end()
            .to_owned()
    }

    /// The text from `start` to `end` inclusive, in reading order. The
    /// points may come in either order and are clamped to the content.
    pub fn selection_text(&self, start: CellPoint, end: CellPoint) -> String {
        let (first, last) = start.in_order(end);

        self.term
            .bounds_to_string(self.engine_point(first), self.engine_point(last))
    }

    fn cursor(&self) -> Cursor {
        let grid = self.term.grid();
        let mut point = grid.cursor.point;

        // On the second half of a wide character the cursor is drawn on
        // the character itself.
        if point.column.0 > 0 && grid[point].flags.contains(Flags::WIDE_CHAR_SPACER) {
            point.column = Column(point.column.0.saturating_sub(1));
        }

        let shape = screen::cursor_shape(self.term.cursor_style().shape);

        Cursor {
            row: usize::try_from(point.line.0).unwrap_or_default(),
            column: point.column.0,
            visible: shape.is_some() && self.term.mode().contains(TermMode::SHOW_CURSOR),
            shape: shape.unwrap_or_default(),
        }
    }

    fn rows(&self, first: usize, count: usize) -> Vec<Row> {
        let end = first.saturating_add(count).min(self.total_lines());
        let grid = self.term.grid();

        (first..end)
            .map(|line| screen::runs(&grid[self.engine_line(line)][..]))
            .collect()
    }

    fn total_lines(&self) -> usize {
        self.history_len().saturating_add(self.screen_lines())
    }

    fn screen_lines(&self) -> usize {
        self.term.screen_lines()
    }

    fn columns(&self) -> usize {
        self.term.columns()
    }

    /// `line` (counted from the oldest scrollback line) in the engine's
    /// numbering, where 0 is the top screen row and history is negative.
    fn engine_line(&self, line: usize) -> Line {
        let line = line.min(self.total_lines().saturating_sub(1));
        let top = i32::try_from(line).unwrap_or(i32::MAX);

        Line(top.saturating_sub(self.history_offset()))
    }

    fn history_offset(&self) -> i32 {
        i32::try_from(self.history_len()).unwrap_or(i32::MAX)
    }

    fn engine_point(&self, point: CellPoint) -> Point {
        Point::new(
            self.engine_line(point.line),
            Column(point.column.min(self.columns().saturating_sub(1))),
        )
    }
}
