//! The rows of a list and the state each one shows.

use anstyle::{AnsiColor, Color};

use super::DIM;

/// A row's state, shown as a coloured sign before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// Running, installed, enabled.
    On,
    /// Stopped, not installed, disabled.
    Off,
    /// Not read.
    Unknown,
    /// Failed or broken.
    Problem,
    /// No state of its own.
    Blank,
}

impl Mark {
    pub(super) const fn sign(self) -> Option<(&'static str, anstyle::Style)> {
        let green = anstyle::Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
        let yellow = anstyle::Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
        let red = anstyle::Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

        match self {
            Self::On => Some(("●", green)),
            Self::Off => Some(("○", DIM)),
            Self::Unknown => Some(("?", yellow)),
            Self::Problem => Some(("✗", red)),
            Self::Blank => None,
        }
    }
}

/// One line of a list: its state and its cells, the first one its name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub mark: Mark,
    pub cells: Vec<String>,
}

impl Row {
    /// Cells hold text from outside too (a packager's description), so
    /// nothing in them may steer the terminal.
    pub fn new(mark: Mark, cells: Vec<String>) -> Self {
        let cells = cells
            .into_iter()
            .map(|cell| toolbox_plan::text::plain(&cell))
            .collect();

        Self { mark, cells }
    }

    /// A row with no state of its own.
    pub fn plain(cells: Vec<String>) -> Self {
        Self::new(Mark::Blank, cells)
    }

    /// A row of borrowed text, as most lists build them.
    pub fn cells(mark: Mark, cells: &[&str]) -> Self {
        Self::new(mark, cells.iter().map(|cell| (*cell).to_owned()).collect())
    }
}
