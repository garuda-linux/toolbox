//! A copy compared with the file in use as byte ranges, so the dialog diffs
//! once per text, not per frame.

use std::ops::Range;

use toolbox_plan::{ConfigText, InvalidConfigText, Resolution};

use super::Pick;
use crate::features::configs::{Change, Diff};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part<'text> {
    Same(&'text str),
    Change {
        current: &'text str,
        copy: &'text str,
    },
}

/// Ranges of whole lines, so each starts and ends on a character boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Stretch {
    /// In the current text. The copy holds the same bytes.
    Same(Range<usize>),
    Change {
        current: Range<usize>,
        copy: Range<usize>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comparison {
    current: String,
    copy: String,
    stretches: Vec<Stretch>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedComparison {
    comparison: Comparison,
    picks: Vec<Pick>,
    resolution: Result<Resolution, InvalidConfigText>,
}

impl Comparison {
    pub fn new(current: String, copy: String) -> Self {
        let stretches = stretches(&current, &copy);

        Self {
            current,
            copy,
            stretches,
        }
    }

    pub fn holds(&self, current: &str, copy: &str) -> bool {
        self.current == current && self.copy == copy
    }

    pub fn parts(&self) -> impl ExactSizeIterator<Item = Part<'_>> {
        self.stretches.iter().map(|stretch| match stretch {
            Stretch::Same(range) => Part::Same(slice(&self.current, range)),
            Stretch::Change { current, copy } => Part::Change {
                current: slice(&self.current, current),
                copy: slice(&self.copy, copy),
            },
        })
    }

    pub fn changes(&self) -> usize {
        changes(&self.stretches)
    }

    /// A change without a pick keeps the current lines.
    pub fn merged(&self, picks: &[Pick]) -> String {
        let mut picked = picks.iter();
        let mut text = String::new();

        for part in self.parts() {
            text.push_str(match part {
                Part::Same(lines) => lines,
                Part::Change { current, copy } => match picked.next() {
                    Some(Pick::Take) => copy,
                    Some(Pick::Keep) | None => current,
                },
            });
        }

        text
    }

    /// # Errors
    ///
    /// [`InvalidConfigText`] for a merged text that is too large.
    pub fn resolution(&self, picks: &[Pick]) -> Result<Resolution, InvalidConfigText> {
        let picked = |wanted: Pick| {
            (0..self.changes()).all(|index| picks.get(index).copied().unwrap_or_default() == wanted)
        };

        if picked(Pick::Keep) {
            return Ok(Resolution::KeepCurrent);
        }

        if picked(Pick::Take) {
            return Ok(Resolution::TakeCopy);
        }

        ConfigText::new(self.merged(picks)).map(|text| Resolution::Merged { text })
    }
}

impl CachedComparison {
    pub fn new(current: String, copy: String) -> Self {
        let comparison = Comparison::new(current, copy);
        let resolution = comparison.resolution(&[]);

        Self {
            comparison,
            picks: Vec::new(),
            resolution,
        }
    }

    pub const fn comparison(&self) -> &Comparison {
        &self.comparison
    }

    pub fn resolution(&mut self, picks: &[Pick]) -> &Result<Resolution, InvalidConfigText> {
        if self.picks != picks {
            self.resolution = self.comparison.resolution(picks);
            picks.clone_into(&mut self.picks);
        }

        &self.resolution
    }
}

/// Groups the lines of [`Diff`] into stretches with byte ranges in both texts.
pub(super) fn stretches(current: &str, copy: &str) -> Vec<Stretch> {
    let mut stretches: Vec<Stretch> = Vec::new();
    let (mut in_current, mut in_copy) = (0, 0);

    for line in Diff::new(current, copy).lines() {
        let length = line.text.len();
        let current_line = in_current..in_current + length;
        let copy_line = in_copy..in_copy + length;

        match (line.change, stretches.last_mut()) {
            (Change::Same, Some(Stretch::Same(range)))
            | (
                Change::Removed,
                Some(Stretch::Change {
                    current: range,
                    copy: _,
                }),
            ) => {
                range.end = current_line.end;
            }
            (
                Change::Added,
                Some(Stretch::Change {
                    current: _,
                    copy: range,
                }),
            ) => {
                range.end = copy_line.end;
            }
            (Change::Same, _) => stretches.push(Stretch::Same(current_line.clone())),
            (Change::Removed, _) => stretches.push(Stretch::Change {
                current: current_line.clone(),
                copy: in_copy..in_copy,
            }),
            (Change::Added, _) => stretches.push(Stretch::Change {
                current: in_current..in_current,
                copy: copy_line.clone(),
            }),
        }

        match line.change {
            Change::Same => (in_current, in_copy) = (current_line.end, copy_line.end),
            Change::Removed => in_current = current_line.end,
            Change::Added => in_copy = copy_line.end,
        }
    }

    stretches
}

pub(super) fn changes(stretches: &[Stretch]) -> usize {
    stretches
        .iter()
        .filter(|stretch| {
            matches!(
                stretch,
                Stretch::Change {
                    current: _,
                    copy: _
                }
            )
        })
        .count()
}

/// Ranges end at line breaks, so `get` cannot miss. Empty is the fallback.
fn slice<'text>(text: &'text str, range: &Range<usize>) -> &'text str {
    text.get(range.clone()).unwrap_or_default()
}
