//! Word and line bounds for double and triple click, from alacritty's own
//! searches, so wide characters and wrapped lines come out right.

#[cfg(test)]
mod tests;

use alacritty_terminal::index::Point;

use crate::{CellPoint, Terminal};

impl Terminal {
    /// The word around `point`: the run of characters between spaces and
    /// the separators alacritty uses (brackets, quotes, `:` and others).
    pub fn word_bounds(&self, point: CellPoint) -> (CellPoint, CellPoint) {
        let point = self.engine_point(point);

        (
            self.cell_point(self.term.semantic_search_left(point)),
            self.cell_point(self.term.semantic_search_right(point)),
        )
    }

    pub fn line_bounds(&self, point: CellPoint) -> (CellPoint, CellPoint) {
        let point = self.engine_point(point);

        (
            self.cell_point(self.term.line_search_left(point)),
            self.cell_point(self.term.line_search_right(point)),
        )
    }

    fn cell_point(&self, point: Point) -> CellPoint {
        let history = i32::try_from(self.history_len()).unwrap_or(i32::MAX);
        let line = point.line.0.saturating_add(history);

        CellPoint {
            line: usize::try_from(line).unwrap_or_default(),
            column: point.column.0,
        }
    }
}
