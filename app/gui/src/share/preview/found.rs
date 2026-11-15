//! The preview's rows and the search's matches. Finding them reads every
//! line, so they are kept while the text, the query and the width stay.

#[cfg(test)]
mod tests;

use toolbox_gui_kit::widgets::matches;
use toolbox_pty::{Run, styled_lines};
use toolbox_services::diagnostics::view::{self, Row};

/// A text is known by where its bytes lie and how many there are, as the
/// Diagnostics report knows its lines.
type Identity = (usize, usize);

#[derive(Clone, Debug, Default)]
pub struct Found {
    text: Identity,
    lines: Vec<Vec<Run>>,
    columns: Option<usize>,
    rows: Vec<Row>,
    query: String,
    /// Each match as its line and the character it starts at.
    hits: Vec<(usize, usize)>,
}

impl Found {
    /// Finds again what `text`, `query` or `columns` changed.
    pub fn update(&mut self, text: &str, query: &str, columns: Option<usize>) {
        let identity = (text.as_ptr().addr(), text.len());
        let new_text = self.text != identity;
        if new_text {
            self.text = identity;
            self.lines = styled_lines(text);
        }

        if new_text || self.columns != columns {
            self.columns = columns;
            self.rows = view::rows(&self.lines, "", columns);
        }

        if new_text || self.query != query {
            query.clone_into(&mut self.query);
            self.hits = hits(&self.lines, query);
        }
    }

    pub const fn rows(&self) -> &[Row] {
        self.rows.as_slice()
    }

    pub fn line(&self, line: usize) -> &[Run] {
        self.lines.get(line).map_or(&[][..], Vec::as_slice)
    }

    pub const fn count(&self) -> usize {
        self.hits.len()
    }

    /// The row that holds match `hit`, counted from 0.
    pub fn row_of(&self, hit: usize) -> Option<usize> {
        let (line, start) = *self.hits.get(hit)?;
        self.rows
            .partition_point(|row| (row.line, row.start) <= (line, start))
            .checked_sub(1)
    }
}

/// Where `query` matches in `lines`, line by line, in reading order.
fn hits(lines: &[Vec<Run>], query: &str) -> Vec<(usize, usize)> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    let mut found = Vec::new();
    for (index, runs) in lines.iter().enumerate() {
        let text: String = runs.iter().map(|run| run.text.as_str()).collect();
        for range in matches(&text, query) {
            let start = text.get(..range.start).unwrap_or_default().chars().count();
            found.push((index, start));
        }
    }

    found
}
