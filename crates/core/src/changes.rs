//! The user's wishes that differ from the system: the queue the Review
//! page shows and Apply turns into a plan. One queue per [`Domain`],
//! created on first use, so new domains need no registration here.

#[cfg(test)]
mod tests;

use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

use toolbox_plan::PlannedAction;

use crate::Snapshot;
use crate::domain::{ChangeView, Domain, Stage, Staged};
use crate::saved::{SAVED_VERSION, SavedChange, SavedChanges};

#[derive(Default)]
pub struct ChangeSet {
    queues: HashMap<TypeId, Box<dyn Queue>>,
}

impl ChangeSet {
    /// Records the wish for `item`, replacing an earlier one. A wish the
    /// system already fulfils removes the item instead, so toggling twice
    /// leaves nothing queued. A wish that does not [`Domain::fits`] the
    /// item changes nothing.
    pub fn want<D>(&mut self, item: D::Item, wish: D::Wish, snapshot: &Snapshot)
    where
        D: Domain,
    {
        if !D::fits(&item, &wish) {
            tracing::warn!("ignored a wish of {} that names another target", D::ID);
            return;
        }

        if is_satisfied::<D>(&item, &wish, snapshot) {
            self.remove::<D>(&item);
        } else {
            self.queue_mut::<D>().changes.insert(item, wish);
        }
    }

    pub fn wish<D>(&self, item: &D::Item) -> Option<&D::Wish>
    where
        D: Domain,
    {
        self.queue::<D>()?.changes.get(item)
    }

    pub fn queued<D>(&self) -> Vec<(D::Item, D::Wish)>
    where
        D: Domain,
    {
        self.queue::<D>().map_or_else(Vec::new, |queue| {
            queue
                .changes
                .iter()
                .map(|(item, wish)| (item.clone(), wish.clone()))
                .collect()
        })
    }

    pub fn remove<D>(&mut self, item: &D::Item)
    where
        D: Domain,
    {
        self.queue_mut::<D>().changes.remove(item);
    }

    /// Returns whether one was queued.
    pub fn remove_key(&mut self, domain: &str, key: &str) -> bool {
        self.queues
            .values_mut()
            .find(|queue| queue.domain() == domain)
            .is_some_and(|queue| queue.remove_key(key))
    }

    /// Drops every change the system now fulfils, after an apply or when
    /// something outside the app changed the system.
    pub fn reconcile(&mut self, snapshot: &Snapshot) {
        for queue in self.queues.values_mut() {
            queue.reconcile(snapshot);
        }
    }

    /// Drops each [`Domain::ONE_OFF`] change whose actions are all in
    /// `done`, the succeeded steps. No probe sees that such work ran, so
    /// after an apply that stopped early this keeps the rest for a retry.
    pub fn drop_done_one_offs(&mut self, done: &[PlannedAction]) {
        for queue in self.queues.values_mut().filter(|queue| queue.one_off()) {
            queue.drop_done(done);
        }
    }

    pub fn clear(&mut self) {
        self.queues.clear();
    }

    pub fn to_saved(&self) -> SavedChanges {
        SavedChanges {
            version: SAVED_VERSION,
            changes: self.in_plan_order().flat_map(Queue::save).collect(),
        }
    }

    pub fn views(&self) -> Vec<ChangeView> {
        self.in_plan_order().flat_map(Queue::views).collect()
    }

    pub fn len(&self) -> usize {
        self.queues.values().map(|queue| queue.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The actions of every domain, sorted by their own stage. Ties keep
    /// the order of the domains (stage, then id) and of each domain's plan.
    pub(crate) fn actions(&self) -> Vec<PlannedAction> {
        let mut staged: Vec<Staged> = self.in_plan_order().flat_map(Queue::plan).collect();

        staged.sort_by_key(|step| step.stage);
        staged.into_iter().map(|step| step.action).collect()
    }

    fn in_plan_order(&self) -> impl Iterator<Item = &dyn Queue> {
        let mut queues: Vec<&dyn Queue> = self.queues.values().map(AsRef::as_ref).collect();

        queues.sort_by_key(|queue| (queue.stage(), queue.domain()));
        queues.into_iter()
    }

    fn queue<D>(&self) -> Option<&DomainQueue<D>>
    where
        D: Domain,
    {
        self.queues
            .get(&TypeId::of::<D>())
            .and_then(|queue| queue.as_any().downcast_ref())
    }

    fn queue_mut<D>(&mut self) -> &mut DomainQueue<D>
    where
        D: Domain,
    {
        let slot = self
            .queues
            .entry(TypeId::of::<D>())
            .or_insert_with(|| Box::new(DomainQueue::<D>::default()));

        slot.as_any_mut().downcast_mut().unwrap_or_else(|| {
            unreachable!("the queue stored under a domain's type id belongs to that domain")
        })
    }
}

impl fmt::Debug for ChangeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.views()).finish()
    }
}

trait Queue: Send + Sync {
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
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct DomainQueue<D>
where
    D: Domain,
{
    changes: BTreeMap<D::Item, D::Wish>,
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
        self.changes
            .retain(|item, wish| !is_satisfied::<D>(item, wish, snapshot));
    }

    /// Planned alone, each change names its own actions.
    fn drop_done(&mut self, done: &[PlannedAction]) {
        self.changes.retain(|item, wish| {
            !D::plan(&[(item, wish)])
                .iter()
                .all(|staged| done.contains(&staged.action))
        });
    }

    fn plan(&self) -> Vec<Staged> {
        if self.changes.is_empty() {
            return Vec::new();
        }

        let changes: Vec<(&D::Item, &D::Wish)> = self.changes.iter().collect();

        D::plan(&changes)
    }

    fn views(&self) -> Vec<ChangeView> {
        self.changes
            .iter()
            .map(|(item, wish)| ChangeView {
                domain: D::ID,
                key: D::key(item),
                summary: D::summary(item, wish),
            })
            .collect()
    }

    fn save(&self) -> Vec<SavedChange> {
        self.changes
            .iter()
            .filter_map(|(item, wish)| {
                let key = D::key(item);
                let saved = |value: serde_json::Result<serde_json::Value>| {
                    value
                        .map_err(|error| {
                            tracing::warn!("skipped unsavable {key} of {}: {error}", D::ID);
                        })
                        .ok()
                };

                let item = saved(serde_json::to_value(item))?;
                let wish = saved(serde_json::to_value(wish))?;

                Some(SavedChange {
                    domain: D::ID.to_owned(),
                    item,
                    wish,
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Unknown state (no probe yet) never counts as fulfilled: the change stays.
fn is_satisfied<D>(item: &D::Item, wish: &D::Wish, snapshot: &Snapshot) -> bool
where
    D: Domain,
{
    D::state(snapshot).is_some_and(|state| D::is_satisfied(item, wish, state))
}
