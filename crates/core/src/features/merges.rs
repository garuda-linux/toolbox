//! A copy compared with the file in use, which is the old side.

mod comparison;
#[cfg(test)]
mod tests;

pub use comparison::{CachedComparison, Comparison, Part};
use toolbox_plan::{
    ConfigText, CopyKind, Fingerprint, InvalidConfigText, MergeChoice, MergeCopy, Resolution,
};

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

pub fn standing(copy: &MergeCopy, current: Option<&str>, copied: &str) -> Standing {
    settled(copy, current, copied).unwrap_or_else(|| Standing::Differs {
        changes: comparison::changes(&comparison::stretches(current.unwrap_or_default(), copied)),
    })
}

/// `None` when the lines differ and need a comparison.
pub fn settled(copy: &MergeCopy, current: Option<&str>, copied: &str) -> Option<Standing> {
    match (copy.kind(), current) {
        (CopyKind::OlderSave, _) => Some(Standing::OlderSave),
        (CopyKind::New | CopyKind::Saved | CopyKind::Orig, None) => Some(Standing::CurrentGone),
        (CopyKind::New | CopyKind::Saved | CopyKind::Orig, Some(current)) => {
            (current == copied).then_some(Standing::Same)
        }
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

/// A merge that is no [`ConfigText`] starts the editor from the file in use.
pub fn edit_start(
    merged: &Result<Resolution, InvalidConfigText>,
    current: Option<&str>,
    copied: &str,
) -> String {
    match merged {
        Ok(Resolution::Merged { text }) => text.as_str().to_owned(),
        Ok(Resolution::TakeCopy) => copied.to_owned(),
        Ok(Resolution::KeepCurrent) | Err(_) => current.unwrap_or_default().to_owned(),
    }
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
