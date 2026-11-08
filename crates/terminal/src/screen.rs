//! What the Console draws: rows of styled runs and the cursor.

#[cfg(test)]
mod tests;

use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::vte::ansi::CursorShape as EngineShape;
use toolbox_pty::{Run, Style};

use crate::color::from_engine;

/// One screen line as runs of equally styled text, left to right.
pub type Row = Vec<Run>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CursorShape {
    #[default]
    Block,
    Underline,
    Beam,
    HollowBlock,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Cursor {
    /// Screen row, 0 at the top.
    pub row: usize,
    /// Cell column, 0 at the left.
    pub column: usize,
    /// Whether to draw it. Full-screen programs often hide it.
    pub visible: bool,
    pub shape: CursorShape,
}

/// The visible screen at one moment, alternate screen included.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Screen {
    /// One entry per screen row, top to bottom.
    pub rows: Vec<Row>,
    /// The cursor, also when the program hid it.
    pub cursor: Cursor,
}

impl Screen {
    pub fn row_texts(&self) -> Vec<String> {
        self.rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|run| run.text.as_str())
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            })
            .collect()
    }
}

/// The engine's shape, `None` for its hidden shape.
pub const fn cursor_shape(shape: EngineShape) -> Option<CursorShape> {
    match shape {
        EngineShape::Block => Some(CursorShape::Block),
        EngineShape::Underline => Some(CursorShape::Underline),
        EngineShape::Beam => Some(CursorShape::Beam),
        EngineShape::HollowBlock => Some(CursorShape::HollowBlock),
        EngineShape::Hidden => None,
    }
}

pub fn runs(cells: &[Cell]) -> Row {
    let mut row: Row = Vec::new();

    for cell in cells {
        // The cell after a wide character only reserves its second column.
        if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
            continue;
        }

        let style = style_of(cell);
        let columns = if cell.flags.contains(Flags::WIDE_CHAR) {
            2
        } else {
            1
        };

        if row.last().is_none_or(|last| last.style != style) {
            row.push(Run {
                text: String::new(),
                columns: 0,
                style,
            });
        }

        if let Some(run) = row.last_mut() {
            run.columns = run.columns.saturating_add(columns);
            push_text(&mut run.text, cell);
        }
    }

    row
}

fn push_text(text: &mut String, cell: &Cell) {
    // Hidden text keeps its cells but shows nothing. A leading spacer
    // marks a wide character wrapped to the next line, and a tab cell
    // stands for the blank it moved over.
    if cell
        .flags
        .intersects(Flags::HIDDEN | Flags::LEADING_WIDE_CHAR_SPACER)
        || cell.c == '\t'
    {
        text.push(' ');
        return;
    }

    text.push(cell.c);
    text.extend(cell.zerowidth().into_iter().flatten());
}

/// How `cell` looks, with inverse video applied to its colours.
const fn style_of(cell: &Cell) -> Style {
    let flags = cell.flags;
    let (fg, bg) = if flags.contains(Flags::INVERSE) {
        (from_engine(cell.bg), from_engine(cell.fg))
    } else {
        (from_engine(cell.fg), from_engine(cell.bg))
    };

    Style {
        fg,
        bg,
        bold: flags.contains(Flags::BOLD),
        italic: flags.contains(Flags::ITALIC),
        underline: flags.intersects(Flags::ALL_UNDERLINES),
        dim: flags.contains(Flags::DIM),
        strikeout: flags.contains(Flags::STRIKEOUT),
    }
}
