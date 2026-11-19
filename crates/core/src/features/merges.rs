//! A copy compared with the file in use, which is the old side.

#[cfg(test)]
mod tests;

use toolbox_plan::{
    ConfigText, CopyKind, Fingerprint, InvalidConfigText, MergeChoice, MergeCopy, Resolution,
};

use super::configs::{Change, Diff};
use crate::domains::ConfigMerges;
use crate::{ChangeSet, Snapshot};

/// What a change between the two files keeps. Every change starts with
/// the current file's lines, so nothing changes without a pick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pick {
    #[default]
    Keep,
    Take,
}

/// One stretch of the comparison, each line with its line break.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part<'text> {
    Same(Vec<&'text str>),
    Change {
        current: Vec<&'text str>,
        copy: Vec<&'text str>,
    },
}

/// How a copy stands against the file it belongs to, for its row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    Differs {
        changes: usize,
    },
    /// The copy holds the same text: only removing it makes sense.
    Same,
    /// The file in use is gone: the copy can take its place or go.
    CurrentGone,
    /// An older `.pacsave.N`, which only goes.
    OlderSave,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comparison<'text> {
    parts: Vec<Part<'text>>,
}

impl<'text> Comparison<'text> {
    pub fn new(current_text: &'text str, copy_text: &'text str) -> Self {
        let mut parts: Vec<Part<'text>> = Vec::new();

        for line in Diff::new(current_text, copy_text).lines() {
            match (line.change, parts.last_mut()) {
                (Change::Same, Some(Part::Same(lines))) => lines.push(line.text),
                (Change::Same, _) => parts.push(Part::Same(vec![line.text])),
                (Change::Removed, Some(Part::Change { current, copy: _ })) => {
                    current.push(line.text);
                }
                (Change::Added, Some(Part::Change { current: _, copy })) => copy.push(line.text),
                (Change::Removed, _) => parts.push(Part::Change {
                    current: vec![line.text],
                    copy: Vec::new(),
                }),
                (Change::Added, _) => parts.push(Part::Change {
                    current: Vec::new(),
                    copy: vec![line.text],
                }),
            }
        }

        Self { parts }
    }

    pub fn parts(&self) -> &[Part<'text>] {
        &self.parts
    }

    pub fn changes(&self) -> usize {
        self.parts
            .iter()
            .filter(|part| {
                matches!(
                    part,
                    Part::Change {
                        current: _,
                        copy: _
                    }
                )
            })
            .count()
    }

    /// The text with the `n`th change as `picks[n]` says, the current
    /// lines for a change without a pick.
    pub fn merged(&self, picks: &[Pick]) -> String {
        let mut picked = picks.iter();
        let mut text = String::new();

        for part in &self.parts {
            let lines = match part {
                Part::Same(lines) => lines,
                Part::Change { current, copy } => match picked.next() {
                    Some(Pick::Take) => copy,
                    Some(Pick::Keep) | None => current,
                },
            };

            text.extend(lines.iter().copied());
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

pub fn standing(copy: &MergeCopy, current: Option<&str>, copied: &str) -> Standing {
    match (copy.kind(), current) {
        (CopyKind::OlderSave, _) => Standing::OlderSave,
        (CopyKind::New | CopyKind::Saved | CopyKind::Orig, None) => Standing::CurrentGone,
        (CopyKind::New | CopyKind::Saved | CopyKind::Orig, Some(current)) if current == copied => {
            Standing::Same
        }
        (CopyKind::New | CopyKind::Saved | CopyKind::Orig, Some(current)) => Standing::Differs {
            changes: Comparison::new(current, copied).changes(),
        },
    }
}

/// A text the user edited by hand, as the resolution it amounts to.
///
/// # Errors
///
/// [`InvalidConfigText`] for a text that is too large or holds a NUL.
pub fn edited(text: &str, current: &str, copied: &str) -> Result<Resolution, InvalidConfigText> {
    if text == current {
        return Ok(Resolution::KeepCurrent);
    }

    if text == copied {
        return Ok(Resolution::TakeCopy);
    }

    ConfigText::new(text).map(|text| Resolution::Merged { text })
}

pub fn choice(resolution: Resolution, current: Option<&str>, copied: &str) -> MergeChoice {
    MergeChoice {
        resolution,
        copy: Fingerprint::of(copied.as_bytes()),
        current: current.map(|current| Fingerprint::of(current.as_bytes())),
    }
}

/// The one way the window and the command line queue a merge.
pub fn queue(changes: &mut ChangeSet, copy: MergeCopy, choice: MergeChoice, snapshot: &Snapshot) {
    changes.want::<ConfigMerges>(copy, choice, snapshot);
}

pub fn queued<'changes>(
    changes: &'changes ChangeSet,
    copy: &MergeCopy,
) -> Option<&'changes MergeChoice> {
    changes.wish::<ConfigMerges>(copy)
}

pub fn take_back(changes: &mut ChangeSet, copy: &MergeCopy) {
    changes.remove::<ConfigMerges>(copy);
}
