//! One domain's queue, behind a trait so the change set holds every
//! domain in one map.

use std::any::Any;
use std::collections::BTreeMap;

use toolbox_plan::PlannedAction;

use crate::Snapshot;
use crate::domain::{ChangeView, Domain, Stage, Staged};
use crate::outcome::Outcome;
use crate::saved::SavedChange;

pub(super) trait Queue: Send + Sync {
    fn domain(&self) -> &'static str;
    fn stage(&self) -> Stage;
    fn one_off(&self) -> bool;
    fn len(&self) -> usize;
    fn reconcile(&mut self, snapshot: &Snapshot);
    fn drop_done(&mut self, done: &[PlannedAction]);
    fn plan(&self) -> Vec<Staged>;
    fn views(&self) -> Vec<ChangeView>;
    fn save(&self) -> Vec<SavedChange>;
    fn remove_key(&mut self, key: &str) -> bool;
    fn set_outcome(&mut self, key: &str, outcome: Outcome);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub(super) struct DomainQueue<D>
where
    D: Domain,
{
    pub(super) changes: BTreeMap<D::Item, Entry<D::Wish>>,
}

pub(super) struct Entry<W> {
    pub(super) wish: W,
    pub(super) outcome: Outcome,
}

impl<W> Entry<W> {
    pub(super) const fn new(wish: W) -> Self {
        Self {
            wish,
            outcome: Outcome::Untried,
        }
    }
}

impl<D> Default for DomainQueue<D>
where
    D: Domain,
{
    fn default() -> Self {
        Self {
            changes: BTreeMap::new(),
        }
    }
}

impl<D> Queue for DomainQueue<D>
where
    D: Domain,
{
    fn domain(&self) -> &'static str {
        D::ID
    }

    fn stage(&self) -> Stage {
        D::STAGE
    }

    fn one_off(&self) -> bool {
        D::ONE_OFF
    }

    fn len(&self) -> usize {
        self.changes.len()
    }

    fn reconcile(&mut self, snapshot: &Snapshot) {
        self.changes.retain(|item, entry| {
            entry.outcome.is_failed() || !is_satisfied::<D>(item, &entry.wish, snapshot)
        });
    }

    /// Planned alone, each change names its own actions.
    fn drop_done(&mut self, done: &[PlannedAction]) {
        self.changes.retain(|item, entry| {
            !D::plan(&[(item, &entry.wish)])
                .iter()
                .all(|staged| done.contains(&staged.action))
        });
    }

    fn plan(&self) -> Vec<Staged> {
        if self.changes.is_empty() {
            return Vec::new();
        }

        let changes: Vec<(&D::Item, &D::Wish)> = self
            .changes
            .iter()
            .map(|(item, entry)| (item, &entry.wish))
            .collect();

        D::plan(&changes)
    }

    fn views(&self) -> Vec<ChangeView> {
        self.changes
            .iter()
            .map(|(item, entry)| ChangeView {
                domain: D::ID,
                key: D::key(item),
                summary: D::summary(item, &entry.wish),
                outcome: entry.outcome,
            })
            .collect()
    }

    fn save(&self) -> Vec<SavedChange> {
        self.changes
            .iter()
            .filter_map(|(item, entry)| {
                let key = D::key(item);
                let saved = |value: serde_json::Result<serde_json::Value>| {
                    value
                        .map_err(|error| {
                            tracing::warn!("skipped unsavable {key} of {}: {error}", D::ID);
                        })
                        .ok()
                };

                let item = saved(serde_json::to_value(item))?;
                let wish = saved(serde_json::to_value(&entry.wish))?;

                Some(SavedChange {
                    domain: D::ID.to_owned(),
                    item,
                    wish,
                    outcome: entry.outcome,
                })
            })
            .collect()
    }

    fn remove_key(&mut self, key: &str) -> bool {
        let found = self
            .changes
            .keys()
            .find(|item| D::key(item) == key)
            .cloned();

        found.is_some_and(|item| self.changes.remove(&item).is_some())
    }

    fn set_outcome(&mut self, key: &str, outcome: Outcome) {
        if let Some(entry) = self
            .changes
            .iter_mut()
            .find_map(|(item, entry)| (D::key(item) == key).then_some(entry))
        {
            entry.outcome = outcome;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Unknown state (no probe yet) never counts as fulfilled: the change stays.
pub(super) fn is_satisfied<D>(item: &D::Item, wish: &D::Wish, snapshot: &Snapshot) -> bool
where
    D: Domain,
{
    D::state(snapshot).is_some_and(|state| D::is_satisfied(item, wish, state))
}
