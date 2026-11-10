//! A queue change sent from a command to the window, over the session bus
//! or into `changes.json` when no window runs.
//!
//! The window reads it from another process, so [`QueueEdit::parse`]
//! checks every value before the queue sees it.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use toolbox_core::{
    ChangeSet, Domains, RestoreProblem, SAVED_VERSION, SavedChange, SavedChanges, Snapshot,
};

/// More than any page queues at once, far below what slows the window.
pub const MAX_EDITS: usize = 512;

/// Keys are names and paths, never longer than this.
const MAX_KEY_BYTES: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueEdit {
    pub version: u32,
    pub remove: Vec<Removal>,
    pub add: Vec<SavedChange>,
}

/// A queued change to take back, as the Review page names it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Removal {
    pub domain: String,
    pub key: String,
}

/// Why a text is no edit the queue takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    Shape,
    Version,
    TooMany,
    Domain,
    Key,
    Change,
}

impl QueueEdit {
    /// What turns `before` into `after`.
    pub fn between(before: &ChangeSet, after: &ChangeSet) -> Self {
        let kept: BTreeSet<Removal> = after.views().into_iter().map(Removal::of).collect();
        let old = before.to_saved().changes;

        Self {
            version: SAVED_VERSION,
            remove: before
                .views()
                .into_iter()
                .map(Removal::of)
                .filter(|removal| !kept.contains(removal))
                .collect(),
            add: after
                .to_saved()
                .changes
                .into_iter()
                .filter(|change| !old.contains(change))
                .collect(),
        }
    }

    pub const fn removing(removals: Vec<Removal>) -> Self {
        Self {
            version: SAVED_VERSION,
            remove: removals,
            add: Vec::new(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.remove.is_empty() && self.add.is_empty()
    }

    /// # Errors
    ///
    /// [`Invalid`] for text from another process that is no edit of a
    /// queue `domains` restores.
    pub fn parse(text: &str, domains: &Domains) -> Result<Self, Invalid> {
        let edit: Self = serde_json::from_str(text).map_err(|_shape| Invalid::Shape)?;

        if edit.version != SAVED_VERSION {
            return Err(Invalid::Version);
        }

        if edit.remove.len() + edit.add.len() > MAX_EDITS {
            return Err(Invalid::TooMany);
        }

        let ids = domains.ids();
        if !edit
            .remove
            .iter()
            .all(|removal| ids.contains(&removal.domain.as_str()))
        {
            return Err(Invalid::Domain);
        }

        if !edit.remove.iter().all(|removal| plain_key(&removal.key)) {
            return Err(Invalid::Key);
        }

        // Decoding each wish into its domain checks its value objects.
        // An empty snapshot fulfils nothing, so every wish is decoded.
        let mut scratch = ChangeSet::default();
        if !edit
            .apply(&mut scratch, domains, &Snapshot::default())
            .is_empty()
        {
            return Err(Invalid::Change);
        }

        Ok(edit)
    }

    /// Takes back the removals and queues the wishes against `snapshot`,
    /// as a click would. Returns what could not be queued.
    pub fn apply(
        &self,
        changes: &mut ChangeSet,
        domains: &Domains,
        snapshot: &Snapshot,
    ) -> Vec<RestoreProblem> {
        for removal in &self.remove {
            changes.remove_key(&removal.domain, &removal.key);
        }

        let added = SavedChanges {
            version: self.version,
            changes: self.add.clone(),
        };

        domains.restore(changes, &added, snapshot)
    }

    /// The edit as the bus and the tests carry it.
    ///
    /// # Errors
    ///
    /// What `serde_json` refuses, which an edit of strings and JSON values
    /// never gives.
    pub fn to_text(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl Removal {
    pub fn of(view: toolbox_core::ChangeView) -> Self {
        Self {
            domain: view.domain.to_owned(),
            key: view.key,
        }
    }
}

impl Invalid {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shape => "the text is no queue edit",
            Self::Version => "the queue edit has another version",
            Self::TooMany => "the queue edit has too many changes",
            Self::Domain => "the queue edit names an unknown kind of change",
            Self::Key => "the queue edit names a change with a control character or too long",
            Self::Change => "the queue edit has a change this app does not take",
        }
    }
}

fn plain_key(key: &str) -> bool {
    (1..=MAX_KEY_BYTES).contains(&key.len()) && !key.chars().any(char::is_control)
}
