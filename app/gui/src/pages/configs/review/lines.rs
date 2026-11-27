//! The review's diff split into rows once per comparison: what each line
//! is, its numbers in the file as it is and as edited, and how wide the
//! widest line is. The view then lays out only the rows in sight, and its
//! sideways extent is known before a long line scrolls into view.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::{Context, Id};
use toolbox_core::features::configs::hunk_lines;

/// egui lays a tab out as this many spaces.
const TAB_COLUMNS: usize = 4;

/// The lines of the edited file a run of changes covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Covered {
    pub first: u32,
    pub last: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The start of a run of lines, said as the lines it covers.
    Hunk(Covered),
    Same,
    Added,
    Removed,
    /// diff's own words, such as the missing line break at the end.
    Note,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub kind: Kind,
    /// The number in the file as it is, `None` for an added line.
    pub old: Option<u32>,
    /// The number in the edited file, `None` for a removed line.
    pub new: Option<u32>,
    /// The line as the plain diff has it, its `+` or `-` included, which a
    /// screen reader hears.
    pub line: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Rows {
    pub source: String,
    pub lines: Vec<Row>,
    /// The most columns a line's text takes, a tab as [`TAB_COLUMNS`].
    pub columns: u16,
    pub digits: u16,
}

impl Row {
    /// The line's text without its mark.
    pub fn text(&self) -> &str {
        match self.kind {
            Kind::Same | Kind::Added | Kind::Removed => self.line.get(1..).unwrap_or_default(),
            Kind::Hunk(_) | Kind::Note => &self.line,
        }
    }
}

impl Rows {
    pub fn parse(plain: &str) -> Self {
        let mut rows = Vec::new();
        let (mut old, mut new) = (0_u32, 0_u32);

        for line in plain.lines() {
            if let (Some((first, last)), Some((old_start, new_start))) =
                (hunk_lines(line), starts(line))
            {
                (old, new) = (old_start, new_start);
                rows.push(Row {
                    kind: Kind::Hunk(Covered { first, last }),
                    old: None,
                    new: None,
                    line: line.to_owned(),
                });
                continue;
            }

            let (kind, numbers) = match line.chars().next() {
                Some('+') => (Kind::Added, (None, Some(next(&mut new)))),
                Some('-') => (Kind::Removed, (Some(next(&mut old)), None)),
                Some(' ') => (Kind::Same, (Some(next(&mut old)), Some(next(&mut new)))),
                Some(_) | None => (Kind::Note, (None, None)),
            };

            rows.push(Row {
                kind,
                old: numbers.0,
                new: numbers.1,
                line: line.to_owned(),
            });
        }

        let widest = rows
            .iter()
            .filter(|row| !matches!(row.kind, Kind::Hunk(_)))
            .map(|row| columns(row.text()))
            .max()
            .unwrap_or_default();

        let highest = rows
            .iter()
            .flat_map(|row| [row.old, row.new])
            .flatten()
            .max()
            .unwrap_or(1);

        Self {
            source: plain.to_owned(),
            lines: rows,
            columns: u16::try_from(widest).unwrap_or(u16::MAX),
            digits: u16::try_from(highest.checked_ilog10().unwrap_or(0) + 1).unwrap_or(u16::MAX),
        }
    }
}

/// The rows of `plain`, split again only when the text differs from the
/// last frame's.
pub fn cached(ctx: &Context, plain: &str) -> Arc<Rows> {
    let id = Id::unique("toolbox_gui::configs::review::rows");
    let kept = ctx.data(|data| data.get_temp::<Arc<Rows>>(id));

    if let Some(rows) = kept.filter(|rows| rows.source == plain) {
        return rows;
    }

    let rows = Arc::new(Rows::parse(plain));
    ctx.data_mut(|data| data.insert_temp(id, Arc::clone(&rows)));
    rows
}

/// The start lines of a hunk header `@@ -3,4 +3,5 @@` in both files.
fn starts(line: &str) -> Option<(u32, u32)> {
    let ranges = line.strip_prefix("@@ -")?;
    let (old, rest) = ranges.split_once(' ')?;
    let new = rest.strip_prefix('+')?.split(' ').next()?;
    let start = |range: &str| range.split(',').next()?.parse::<u32>().ok();

    Some((start(old)?, start(new)?))
}

const fn next(number: &mut u32) -> u32 {
    let this = *number;
    *number = this.saturating_add(1);
    this
}

fn columns(text: &str) -> usize {
    text.chars()
        .map(|character| if character == '\t' { TAB_COLUMNS } else { 1 })
        .sum()
}
