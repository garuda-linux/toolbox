//! The review's diff split into rows once per comparison: what each line
//! is, its numbers in the file as it is and as edited, and how wide the
//! widest line is. The view then lays out only the rows in sight, and its
//! sideways extent is known before a long line scrolls into view.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::{Context, Id};
use toolbox_core::features::configs::diff::{Kind, Row, parse_rows};
use toolbox_text::OutsideText;

/// egui lays a tab out as this many spaces.
const TAB_COLUMNS: usize = 4;

#[derive(Debug, PartialEq, Eq)]
pub struct Rows {
    pub source: OutsideText,
    pub lines: Vec<Row>,
    /// The most columns a line's text takes, a tab as [`TAB_COLUMNS`].
    pub columns: u16,
    pub digits: u16,
}

impl Rows {
    pub fn parse(plain: &OutsideText) -> Self {
        let lines = parse_rows(plain);

        let widest = lines
            .iter()
            .filter(|row| !matches!(row.kind, Kind::Hunk(_)))
            .map(|row| columns(row.text()))
            .max()
            .unwrap_or_default();

        let highest = lines
            .iter()
            .flat_map(|row| [row.old, row.new])
            .flatten()
            .max()
            .unwrap_or(1);

        Self {
            source: plain.clone(),
            lines,
            columns: u16::try_from(widest).unwrap_or(u16::MAX),
            digits: u16::try_from(highest.checked_ilog10().unwrap_or(0) + 1).unwrap_or(u16::MAX),
        }
    }
}

/// The rows of `plain`, split again only when the text differs from the
/// last frame's.
pub fn cached(ctx: &Context, plain: &OutsideText) -> Arc<Rows> {
    let id = Id::unique("toolbox_gui::configs::review::rows");
    let kept = ctx.data(|data| data.get_temp::<Arc<Rows>>(id));

    if let Some(rows) = kept.filter(|rows| rows.source == *plain) {
        return rows;
    }

    let rows = Arc::new(Rows::parse(plain));
    ctx.data_mut(|data| data.insert_temp(id, Arc::clone(&rows)));
    rows
}

fn columns(text: &str) -> usize {
    text.chars()
        .map(|character| if character == '\t' { TAB_COLUMNS } else { 1 })
        .sum()
}
