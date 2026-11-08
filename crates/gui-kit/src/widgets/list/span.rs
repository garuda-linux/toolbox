//! Which rows of a list to lay out, after spotifast's `virtual_rows`:
//! those that meet the visible part of the scroll view, one more on each
//! side so Tab reaches it and scrolls it in, and the focused one wherever
//! it is, so it keeps the focus.

use egui::Rangef;

use super::super::card::DIVIDER;

#[derive(Clone, Debug, PartialEq)]
pub enum Offsets {
    /// Every row as tall as the others.
    Fixed { height: f32, count: usize },
    /// The bottom of each row, the heights added up.
    Measured { bottoms: Vec<f32> },
}

impl Offsets {
    pub fn measured(heights: impl IntoIterator<Item = f32>) -> Self {
        let mut total = 0.0;
        let bottoms = heights
            .into_iter()
            .map(|height| {
                total += height;
                total
            })
            .collect();

        Self::Measured { bottoms }
    }

    pub const fn count(&self) -> usize {
        match self {
            Self::Fixed { height: _, count } => *count,
            Self::Measured { bottoms } => bottoms.len(),
        }
    }

    /// The top of row `index`, or the list's bottom past its last row.
    pub fn top(&self, index: usize) -> f32 {
        match self {
            // Every row but the first has its divider's pixel above it.
            Self::Fixed { height, count } => {
                let index = index.min(*count);
                height.mul_add(points(index), DIVIDER * points(index.saturating_sub(1)))
            }
            Self::Measured { bottoms } => index
                .checked_sub(1)
                .and_then(|above| bottoms.get(above.min(bottoms.len().saturating_sub(1))))
                .copied()
                .unwrap_or(0.0),
        }
    }

    pub fn total(&self) -> f32 {
        self.top(self.count())
    }

    /// The row at `offset`, clamped to the list.
    pub fn index_at(&self, offset: f32) -> usize {
        match self {
            Self::Fixed { height, count } => {
                if *height <= 0.0 || offset <= 0.0 {
                    return 0;
                }

                let rows = ((offset + DIVIDER) / (height + DIVIDER)).floor();
                whole(rows).min(*count)
            }
            Self::Measured { bottoms } => bottoms.partition_point(|bottom| *bottom <= offset),
        }
    }

    /// The rows to lay out while `visible` (from the list's top) shows,
    /// in order. `keep` is laid out wherever it is.
    pub fn rows_to_draw(&self, visible: Rangef, keep: Option<usize>) -> Vec<usize> {
        let count = self.count();
        let first = self.index_at(visible.min).min(count).saturating_sub(1);

        // The rows that start above the view's bottom, then one more.
        let end = if visible.max > 0.0 {
            self.index_at(visible.max).saturating_add(1)
        } else {
            0
        };

        let last = end.saturating_add(1).min(count);
        let mut rows: Vec<usize> = (first..last).collect();
        if let Some(keep) = keep.filter(|keep| *keep < count && !(first..last).contains(keep)) {
            let at = rows.partition_point(|row| *row < keep);
            rows.insert(at, keep);
        }

        rows
    }
}

#[expect(
    clippy::cast_precision_loss,
    clippy::as_conversions,
    reason = "no list is long enough for a float to misplace a row"
)]
const fn points(index: usize) -> f32 {
    index as f32
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::as_conversions,
    reason = "the value is floored and not negative"
)]
const fn whole(rows: f32) -> usize {
    rows as usize
}
