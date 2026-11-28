//! The lines of a plain diff as the review lays them out: what each line
//! is and its number in the file as it is and as edited.

use toolbox_text::OutsideText;

use super::hunk_lines;

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
    pub line: OutsideText,
}

impl Row {
    /// The line's text without its mark.
    pub fn text(&self) -> &str {
        match self.kind {
            Kind::Same | Kind::Added | Kind::Removed => self.line.get(1..).unwrap_or_default(),
            Kind::Hunk(_) | Kind::Note => self.line.as_str(),
        }
    }
}

/// The rows of a plain diff from [`super::Diff::plain`], each numbered in
/// both files.
pub fn parse_rows(plain: &OutsideText) -> Vec<Row> {
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
                line: OutsideText::keep_lines(line),
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
            line: OutsideText::keep_lines(line),
        });
    }

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
