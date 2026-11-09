//! Queued changes outlive the app: they are saved as plain JSON after every
//! change and restored on the next start. Restoring needs the domain type
//! behind each saved id, which [`Domains`] provides: one `with::<D>()` line
//! per domain.

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::Domain;
use crate::{ChangeSet, Outcome, Snapshot};

/// Bumped when the file's shape changes. Older files are then dropped
/// instead of misread.
pub const SAVED_VERSION: u32 = 5;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedChanges {
    pub version: u32,
    pub changes: Vec<SavedChange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedChange {
    pub domain: String,
    pub item: Value,
    pub wish: Value,
    /// Left out while untried, so a file without it reads as before.
    #[serde(default, skip_serializing_if = "Outcome::is_untried")]
    pub outcome: Outcome,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RestoreProblem {
    #[error("the saved changes have version {0}, this app reads version {SAVED_VERSION}")]
    Version(u32),
    #[error("no domain `{0}` exists anymore")]
    UnknownDomain(String),
    #[error("a saved `{domain}` change is invalid: {reason}")]
    Invalid { domain: String, reason: String },
}

/// Two domains with one id: the saved changes of one would come back as
/// the other's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("two domains share the id `{0}`")]
pub struct DuplicateDomain(pub &'static str);

type Restore = fn(&mut ChangeSet, &SavedChange, &Snapshot) -> Result<(), RestoreProblem>;

#[derive(Default)]
pub struct Domains {
    restorers: HashMap<&'static str, Restore>,
}

impl Domains {
    /// # Errors
    ///
    /// When a domain with the same [`Domain::ID`] is registered already.
    pub fn with<D>(mut self) -> Result<Self, DuplicateDomain>
    where
        D: Domain,
    {
        match self.restorers.insert(D::ID, restore::<D>) {
            None => Ok(self),
            Some(_) => Err(DuplicateDomain(D::ID)),
        }
    }

    /// The ids of the domains, sorted, so a list can be checked against
    /// them, such as the command line's.
    pub fn ids(&self) -> Vec<&'static str> {
        let mut ids: Vec<&'static str> = self.restorers.keys().copied().collect();
        ids.sort_unstable();
        ids
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
                self.restorers.get(change.domain.as_str()).map_or_else(
                    || Some(RestoreProblem::UnknownDomain(change.domain.clone())),
                    |restore| restore(changes, change, snapshot).err(),
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
    let invalid = |error: serde_json::Error| RestoreProblem::Invalid {
        domain: saved.domain.clone(),
        reason: error.to_string(),
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
