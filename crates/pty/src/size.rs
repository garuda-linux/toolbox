//! The terminal size a program sees.

#[cfg(test)]
mod tests;

use rustix::termios::Winsize;

/// Used until the Console reports its size. Wide enough for pacman's
/// package tables.
pub const DEFAULT_SIZE: TerminalSize = TerminalSize {
    columns: 140,
    rows: 40,
};

const MIN_COLUMNS: u16 = 40;
const MAX_COLUMNS: u16 = 400;
const MIN_ROWS: u16 = 10;
const MAX_ROWS: u16 = 200;

/// Columns and rows in cells. A local type, so `toolbox-pty` links no
/// `toolbox-plan`: the protocol's own size stays where the helper reads
/// it. Convert with [`From`], which the orphan rule allows for tuples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalSize {
    pub columns: u16,
    pub rows: u16,
}

impl From<(u16, u16)> for TerminalSize {
    fn from((columns, rows): (u16, u16)) -> Self {
        Self { columns, rows }
    }
}

/// `size` within bounds programs can draw in. A Console squeezed to a
/// few cells would make pacman wrap every line. The upper bounds keep a
/// bogus size from the app harmless.
pub fn clamped(size: TerminalSize) -> TerminalSize {
    TerminalSize {
        columns: size.columns.clamp(MIN_COLUMNS, MAX_COLUMNS),
        rows: size.rows.clamp(MIN_ROWS, MAX_ROWS),
    }
}

/// `size` as the kernel's window size. A plain function because the orphan
/// rule forbids `From` between two foreign types.
pub const fn winsize(size: TerminalSize) -> Winsize {
    Winsize {
        ws_row: size.rows,
        ws_col: size.columns,
        ws_xpixel: 0,
        ws_ypixel: 0,
    }
}
