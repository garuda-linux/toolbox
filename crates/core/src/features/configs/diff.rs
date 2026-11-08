//! What an edit changed, line by line: the text the user sees before the
//! helper writes a config file back.

#[cfg(test)]
mod tests;

use std::fmt::Write as _;

/// Unchanged lines shown around a change, as `diff -u` does.
const CONTEXT: usize = 3;

/// Above this many cells the table of the comparison is not built: the
/// changed middle is then shown as all removed and all added, which is
/// still correct.
const MAX_CELLS: usize = 4_000_000;

const ADDED: &str = "\u{1b}[32m";
const REMOVED: &str = "\u{1b}[31m";
const HEADER: &str = "\u{1b}[36m";
const RESET: &str = "\u{1b}[0m";

/// Stands in for a control character of the file, which would otherwise
/// be read as a part of the colour codes around the line.
const REPLACEMENT: char = '\u{fffd}';

/// Said for a last line without a line break, like `diff` does.
const NO_LINE_BREAK: &str = "\\ No line break at the end of the file";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Same,
    Added,
    Removed,
}

/// One line of the old or the new text, with its line break when it has
/// one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line<'text> {
    pub change: Change,
    pub text: &'text str,
}

/// Every line of both texts in reading order: removed lines before the
/// added ones that replace them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diff<'text> {
    lines: Vec<Line<'text>>,
}

/// A run of lines to show, with where it starts in both texts.
struct Hunk<'diff, 'text> {
    old_start: usize,
    new_start: usize,
    lines: &'diff [Line<'text>],
}

impl<'text> Diff<'text> {
    pub fn new(old: &'text str, new: &'text str) -> Self {
        let before: Vec<&str> = old.split_inclusive('\n').collect();
        let after: Vec<&str> = new.split_inclusive('\n').collect();
        let prefix = before
            .iter()
            .zip(&after)
            .take_while(|(old_line, new_line)| old_line == new_line)
            .count();

        let (same_start, old_rest) = before.split_at(prefix);
        let new_rest = after.get(prefix..).unwrap_or_default();
        let suffix = old_rest
            .iter()
            .rev()
            .zip(new_rest.iter().rev())
            .take_while(|(old_line, new_line)| old_line == new_line)
            .count();

        let (old_middle, same_end) = old_rest.split_at(old_rest.len() - suffix);
        let new_middle = new_rest.get(..new_rest.len() - suffix).unwrap_or_default();
        let same = |text: &&'text str| Line {
            change: Change::Same,
            text,
        };

        let mut lines: Vec<Line<'text>> = same_start.iter().map(same).collect();

        lines.extend(middle(old_middle, new_middle));
        lines.extend(same_end.iter().map(same));
        Self { lines }
    }

    #[cfg(test)]
    fn lines(&self) -> &[Line<'text>] {
        &self.lines
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.lines.iter().all(|line| line.change == Change::Same)
    }

    pub fn added(&self) -> usize {
        self.count(Change::Added)
    }

    pub fn removed(&self) -> usize {
        self.count(Change::Removed)
    }

    /// The changes with a few lines around each, `+` and `-` in front,
    /// like `diff -u` prints them. Plain text for the screen reader, copy
    /// and themes without colours.
    pub fn plain(&self) -> String {
        self.render(false)
    }

    /// [`Diff::plain`] with terminal colour codes around added and removed
    /// lines. Control characters of the file are replaced, so nothing of
    /// its content can act as a code.
    pub fn colored(&self) -> String {
        self.render(true)
    }

    fn count(&self, change: Change) -> usize {
        self.lines
            .iter()
            .filter(|line| line.change == change)
            .count()
    }

    fn render(&self, colors: bool) -> String {
        let mut text = String::new();

        for hunk in self.hunks() {
            let (old_count, new_count) = counts(hunk.lines);
            let (open, close) = if colors { (HEADER, RESET) } else { ("", "") };

            // Writing to a String cannot fail.
            writeln!(
                text,
                "{open}@@ -{},{old_count} +{},{new_count} @@{close}",
                hunk.old_start, hunk.new_start
            )
            .ok();

            for line in hunk.lines {
                push_line(&mut text, *line, colors);
            }
        }

        text
    }

    /// The changed lines with [`CONTEXT`] unchanged ones around them.
    /// Changes closer than twice the context share a hunk.
    fn hunks(&self) -> Vec<Hunk<'_, 'text>> {
        let changed: Vec<usize> = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.change != Change::Same)
            .map(|(index, _)| index)
            .collect();

        let mut ranges: Vec<(usize, usize)> = Vec::new();

        for index in changed {
            let start = index.saturating_sub(CONTEXT);
            let end = (index + CONTEXT + 1).min(self.lines.len());

            match ranges.last_mut() {
                Some((_, last_end)) if start <= *last_end => *last_end = end,
                Some(_) | None => ranges.push((start, end)),
            }
        }

        ranges
            .into_iter()
            .filter_map(|(start, end)| {
                let (old_before, new_before) = counts(self.lines.get(..start)?);

                Some(Hunk {
                    old_start: 1 + old_before,
                    new_start: 1 + new_before,
                    lines: self.lines.get(start..end)?,
                })
            })
            .collect()
    }
}

fn counts(lines: &[Line<'_>]) -> (usize, usize) {
    let old = lines
        .iter()
        .filter(|line| line.change != Change::Added)
        .count();

    let new = lines
        .iter()
        .filter(|line| line.change != Change::Removed)
        .count();

    (old, new)
}

/// The lines between the common start and end: a longest common
/// subsequence keeps what both texts share, the rest is removed or added.
fn middle<'text>(old: &[&'text str], new: &[&'text str]) -> Vec<Line<'text>> {
    let removed = |text: &&'text str| Line {
        change: Change::Removed,
        text,
    };

    let added = |text: &&'text str| Line {
        change: Change::Added,
        text,
    };

    let Some(table) = lengths(old, new) else {
        return old
            .iter()
            .map(removed)
            .chain(new.iter().map(added))
            .collect();
    };

    let width = new.len() + 1;
    let at = |row: usize, column: usize| cell(&table, width, row, column);
    let mut lines = Vec::with_capacity(old.len() + new.len());
    let (mut row, mut column) = (0, 0);

    while let (Some(old_line), Some(new_line)) = (old.get(row), new.get(column)) {
        if old_line == new_line {
            lines.push(Line {
                change: Change::Same,
                text: old_line,
            });
            row += 1;
            column += 1;
        } else if at(row + 1, column) >= at(row, column + 1) {
            lines.push(removed(old_line));
            row += 1;
        } else {
            lines.push(added(new_line));
            column += 1;
        }
    }

    lines.extend(old.iter().skip(row).map(removed));
    lines.extend(new.iter().skip(column).map(added));
    lines
}

/// The table of common subsequence lengths, filled from the end: the cell
/// of `(row, column)` is the longest common subsequence of the lines from
/// there on. `None` when it would be too large.
fn lengths(old: &[&str], new: &[&str]) -> Option<Vec<u32>> {
    let width = new.len() + 1;
    let cells = (old.len() + 1).checked_mul(width)?;

    if cells > MAX_CELLS {
        return None;
    }

    let mut table = vec![0_u32; cells];

    for (row, old_line) in old.iter().enumerate().rev() {
        for (column, new_line) in new.iter().enumerate().rev() {
            let value = if old_line == new_line {
                cell(&table, width, row + 1, column + 1) + 1
            } else {
                cell(&table, width, row + 1, column).max(cell(&table, width, row, column + 1))
            };

            if let Some(slot) = table.get_mut(row * width + column) {
                *slot = value;
            }
        }
    }

    Some(table)
}

/// The cell of `(row, column)`, 0 past the edge of the table.
fn cell(table: &[u32], width: usize, row: usize, column: usize) -> u32 {
    table.get(row * width + column).copied().unwrap_or(0)
}

fn push_line(text: &mut String, line: Line<'_>, colors: bool) {
    let (mark, color) = match line.change {
        Change::Same => (' ', ""),
        Change::Added => ('+', ADDED),
        Change::Removed => ('-', REMOVED),
    };

    let (content, ended) = line
        .text
        .strip_suffix('\n')
        .map_or((line.text, false), |content| (content, true));

    let colored = colors && !color.is_empty();

    if colored {
        text.push_str(color);
    }

    text.push(mark);
    text.extend(content.chars().map(|character| {
        if colors && character.is_control() && character != '\t' {
            REPLACEMENT
        } else {
            character
        }
    }));

    if colored {
        text.push_str(RESET);
    }

    text.push('\n');

    if !ended {
        text.push_str(NO_LINE_BREAK);
        text.push('\n');
    }
}

/// The first and last line of the new text a hunk header of
/// [`Diff::plain`] covers, so a review can say "Lines 3 to 6" instead of
/// showing diff markup. A hunk that only removes covers the line before.
pub fn hunk_lines(line: &str) -> Option<(u32, u32)> {
    let new = line
        .strip_prefix("@@ -")?
        .split(' ')
        .nth(1)?
        .strip_prefix('+')?;

    let (start, count) = new.split_once(',')?;
    let (start, count) = (start.parse::<u32>().ok()?, count.parse::<u32>().ok()?);
    let last = start.saturating_add(count.saturating_sub(1));
    Some((start.max(1), last.max(start.max(1))))
}
