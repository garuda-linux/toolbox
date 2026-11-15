//! The rows the report viewer draws.
//!
//! The lines that hold the search, each cut into pieces of one width while
//! the lines wrap. Every row is one line tall, so the viewer lays out only
//! the rows in view, however long the report.

#[cfg(test)]
mod tests;

use toolbox_pty::Run;

/// One piece of report line `line`, from character `start` up to `end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

/// The rows of `lines` that hold `query` (any case, all of them while it
/// is blank), cut every `columns` characters when that is `Some`.
pub fn rows(lines: &[Vec<Run>], query: &str, columns: Option<usize>) -> Vec<Row> {
    let needle = query.trim().to_lowercase();
    let width = columns.filter(|columns| *columns > 0);
    let mut rows = Vec::new();

    for (line, runs) in lines.iter().enumerate() {
        let text: String = runs.iter().map(|run| run.text.as_str()).collect();
        if !needle.is_empty() && !text.to_lowercase().contains(&needle) {
            continue;
        }

        let length = text.chars().count();
        let Some(cut) = width.filter(|width| length > *width) else {
            rows.push(Row {
                line,
                start: 0,
                end: length,
            });
            continue;
        };

        let mut start = 0;
        while start < length {
            let end = start.saturating_add(cut).min(length);
            rows.push(Row { line, start, end });
            start = end;
        }
    }

    rows
}

/// The parts of `runs` between characters `start` and `end`, each with the
/// run it comes from, for its colour.
pub fn piece(runs: &[Run], start: usize, end: usize) -> Vec<(String, &Run)> {
    let mut parts = Vec::new();
    let mut offset = 0_usize;

    for run in runs {
        let count = run.text.chars().count();
        let next = offset.saturating_add(count);
        if next > start && offset < end {
            let skip = start.saturating_sub(offset);
            let take = end.min(next).saturating_sub(offset.max(start));
            let text: String = run.text.chars().skip(skip).take(take).collect();
            parts.push((text, run));
        }

        offset = next;
    }

    parts
}
