//! Queued changes outlive the app: they are saved as plain JSON after every
//! change and restored on the next start. Restoring needs the domain type
//! behind each saved id, which [`Domains`] provides: one `with::<D>()` line
//! per domain.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{Domain, Stage};
use crate::{ChangeSet, Outcome, Snapshot, SummaryCode};

/// Bumped when the file's shape changes. Older files are then dropped
/// instead of misread.
pub const SAVED_VERSION: u32 = 6;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedChanges {
    pub version: u32,
    pub changes: Vec<SavedChange>,
}

#[derive(Deserialize)]
struct Head {
    version: u32,
}

impl SavedChanges {
    /// # Errors
    ///
    /// [`RestoreProblem::Version`] for a file of another version, checked
    /// before the shape, which another version may have changed.
    /// [`RestoreProblem::Unreadable`] for text that is no saved queue.
    pub fn parse(text: &str) -> Result<Self, RestoreProblem> {
        let unreadable = |error: serde_json::Error| RestoreProblem::Unreadable(error.to_string());
        let Head { version } = serde_json::from_str(text).map_err(unreadable)?;
        if version != SAVED_VERSION {
            return Err(RestoreProblem::Version(version));
        }

        serde_json::from_str(text).map_err(unreadable)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedChange {
    pub domain: String,
    pub item: Value,
    pub wish: Value,
    /// Left out while untried, so a file without it reads as before.
    #[serde(default, skip_serializing_if = "Outcome::is_untried")]
    pub outcome: Outcome,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum RestoreProblem {
    #[error("the saved changes have version {0}, this app reads version {SAVED_VERSION}")]
    Version(u32),
    #[error("the saved changes cannot be read: {0}")]
    Unreadable(String),
    #[error("no domain `{0}` exists anymore")]
    UnknownDomain(String),
    #[error("a saved `{domain}` change is invalid: {reason}")]
    Invalid { domain: String, reason: String },
    /// The item or wish does not decode. Shared so the problem stays
    /// `Clone`.
    #[error("a saved `{domain}` change is invalid: {error}")]
    Malformed {
        domain: String,
        #[source]
        error: Arc<serde_json::Error>,
    },
}

/// `serde_json`'s errors have no equality, so two count as equal when they
/// read the same.
impl PartialEq for RestoreProblem {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
            && std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for RestoreProblem {}

/// Two domains the registry cannot keep apart. A shared id would send the
/// saved changes of one back as the other's, and the planner orders by
/// stage alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    #[error("two domains share the id `{0}`")]
    DuplicateDomain(&'static str),
    #[error("two domains run at the stage of rank {0}")]
    SharedStage(u16),
    #[error("two domains give the change code `{0:?}`")]
    SharedCode(SummaryCode),
}

type Restore = fn(&mut ChangeSet, &SavedChange, &Snapshot) -> Result<(), RestoreProblem>;

/// What the registry keeps of one domain type.
struct Entry {
    restore: Restore,
    codes: &'static [SummaryCode],
    stage: Stage,
}

#[derive(Default)]
pub struct Domains {
    entries: HashMap<&'static str, Entry>,
}

impl Domains {
    /// # Errors
    ///
    /// When a registered domain has the same [`Domain::ID`] or
    /// [`Domain::STAGE`], or gives one of its [`Domain::CODES`].
    pub fn with<D>(mut self) -> Result<Self, RegistryError>
    where
        D: Domain,
    {
        if self.entries.contains_key(D::ID) {
            return Err(RegistryError::DuplicateDomain(D::ID));
        }

        if self
            .entries
            .values()
            .any(|entry| entry.stage.rank() == D::STAGE.rank())
        {
            return Err(RegistryError::SharedStage(D::STAGE.rank()));
        }

        if let Some(code) = D::CODES.iter().find(|code| self.covers(**code)) {
            return Err(RegistryError::SharedCode(*code));
        }

        let entry = Entry {
            restore: restore::<D>,
            codes: D::CODES,
            stage: D::STAGE,
        };

        self.entries.insert(D::ID, entry);
        Ok(self)
    }

    /// The ids of the domains, sorted, so a list can be checked against
    /// them, such as the command line's.
    pub fn ids(&self) -> Vec<&'static str> {
        let mut ids: Vec<&'static str> = self.entries.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    /// Whether a registered domain gives `code`.
    pub fn covers(&self, code: SummaryCode) -> bool {
        self.entries
            .values()
            .any(|entry| entry.codes.contains(&code))
    }

    /// Queues every saved change again, compared with `snapshot` like a new
    /// wish, apart from a failed one, which stays. Problems skip the
    /// change and are returned, never fatal.
    pub fn restore(
        &self,
        changes: &mut ChangeSet,
        saved: &SavedChanges,
        snapshot: &Snapshot,
    ) -> Vec<RestoreProblem> {
        if saved.version != SAVED_VERSION {
            return vec![RestoreProblem::Version(saved.version)];
        }

        saved
            .changes
            .iter()
            .filter_map(|change| {
                self.entries.get(change.domain.as_str()).map_or_else(
                    || Some(RestoreProblem::UnknownDomain(change.domain.clone())),
                    |entry| (entry.restore)(changes, change, snapshot).err(),
                )
            })
            .collect()
    }
}

fn restore<D>(
    changes: &mut ChangeSet,
    saved: &SavedChange,
    snapshot: &Snapshot,
) -> Result<(), RestoreProblem>
where
    D: Domain,
{
    let invalid = |error: serde_json::Error| RestoreProblem::Malformed {
        domain: saved.domain.clone(),
        error: Arc::new(error),
    };

    let item: D::Item = serde_json::from_value(saved.item.clone()).map_err(invalid)?;
    let wish: D::Wish = serde_json::from_value(saved.wish.clone()).map_err(invalid)?;

    if !D::fits(&item, &wish) {
        return Err(RestoreProblem::Invalid {
            domain: saved.domain.clone(),
            reason: "the wish names another target than the item".to_owned(),
        });
    }

    changes.restore::<D>(item, wish, saved.outcome, snapshot);
    Ok(())
}
