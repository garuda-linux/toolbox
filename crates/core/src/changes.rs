//! The user's wishes that differ from the system: the queue the Review
//! page shows and Apply turns into a plan. One queue per [`Domain`],
//! created on first use, so new domains need no registration here.

mod queue;
#[cfg(test)]
mod tests;

use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;

use queue::{DomainQueue, Entry, Queue, is_satisfied};
use toolbox_plan::{PlannedAction, StepId};

use crate::Snapshot;
use crate::domain::{ChangeView, Domain, Staged};
use crate::outcome::{Outcome, Owners};
use crate::saved::{SAVED_VERSION, SavedChanges};

#[derive(Default)]
pub struct ChangeSet {
    queues: HashMap<TypeId, Box<dyn Queue>>,
}

impl ChangeSet {
    /// Records the wish for `item`, replacing an earlier one and its outcome.
    /// A wish the system fulfils removes the item, so toggling twice leaves
    /// nothing queued. One that does not [`Domain::fits`] changes nothing.
    pub(crate) fn want<D>(&mut self, item: D::Item, wish: D::Wish, snapshot: &Snapshot)
    where
        D: Domain,
    {
        if !D::fits(&item, &wish) {
            tracing::warn!("ignored a wish of {} that names another target", D::ID);
            return;
        }

        if is_satisfied::<D>(&item, &wish, snapshot) {
            self.remove::<D>(&item);
            return;
        }

        if let Some(queue) = self.queue_mut::<D>() {
            queue.changes.insert(item, Entry::new(wish));
        }
    }

    /// A saved change. A failed one comes back even when the system shows
    /// its wish, as its plan did not finish.
    pub(crate) fn restore<D>(
        &mut self,
        item: D::Item,
        wish: D::Wish,
        outcome: Outcome,
        snapshot: &Snapshot,
    ) where
        D: Domain,
    {
        if outcome.is_failed() {
            if let Some(queue) = self.queue_mut::<D>() {
                queue.changes.insert(item, Entry { wish, outcome });
            }

            return;
        }

        self.want::<D>(item.clone(), wish, snapshot);
        if let Some(entry) = self
            .queue_mut::<D>()
            .and_then(|queue| queue.changes.get_mut(&item))
        {
            entry.outcome = outcome;
        }
    }

    pub fn wish<D>(&self, item: &D::Item) -> Option<&D::Wish>
    where
        D: Domain,
    {
        self.queue::<D>()?
            .changes
            .get(item)
            .map(|entry| &entry.wish)
    }

    pub fn queued<D>(&self) -> Vec<(D::Item, D::Wish)>
    where
        D: Domain,
    {
        self.queue::<D>().map_or_else(Vec::new, |queue| {
            queue
                .changes
                .iter()
                .map(|(item, entry)| (item.clone(), entry.wish.clone()))
                .collect()
        })
    }

    pub(crate) fn remove<D>(&mut self, item: &D::Item)
    where
        D: Domain,
    {
        if let Some(queue) = self.queue_mut::<D>() {
            queue.changes.remove(item);
        }
    }

    /// Returns whether one was queued.
    pub fn remove_key(&mut self, domain: &str, key: &str) -> bool {
        self.queue_of_mut(domain)
            .is_some_and(|queue| queue.remove_key(key))
    }

    /// Drops every change the system now fulfils, after an apply or when
    /// something outside the app changed the system. A change whose last
    /// apply failed stays: its first step may show the wish while a later
    /// one never ran.
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

    /// Records how the plan `owners` came with ended for each change it
    /// carried out. `ended_well` names the steps that succeeded or found
    /// nothing to do.
    pub fn finished(&mut self, owners: &Owners, ended_well: impl Fn(StepId) -> bool) {
        for (domain, keys, outcome) in owners.outcomes(ended_well) {
            if let Some(queue) = self.queue_of_mut(domain) {
                for key in keys {
                    queue.set_outcome(key, outcome);
                }
            }
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

    /// Whether a change of the domain with the id `domain` is queued.
    pub fn holds(&self, domain: &str) -> bool {
        self.queues
            .values()
            .any(|queue| queue.domain() == domain && queue.len() > 0)
    }

    /// The actions of every domain with the domain's id, sorted by their
    /// own stage. Ties keep the order of the domains (stage, then id) and
    /// of each domain's plan.
    pub(crate) fn actions(&self) -> Vec<(&'static str, PlannedAction)> {
        let mut staged: Vec<(&'static str, Staged)> = self
            .in_plan_order()
            .flat_map(|queue| {
                let domain = queue.domain();
                queue.plan().into_iter().map(move |step| (domain, step))
            })
            .collect();

        staged.sort_by_key(|(_, step)| step.stage);
        staged
            .into_iter()
            .map(|(domain, step)| (domain, step.action))
            .collect()
    }

    pub(crate) fn keys(&self, domain: &str) -> Vec<String> {
        self.queues
            .values()
            .filter(|queue| queue.domain() == domain)
            .flat_map(|queue| queue.views())
            .map(|change| change.key)
            .collect()
    }

    fn in_plan_order(&self) -> impl Iterator<Item = &dyn Queue> {
        let mut queues: Vec<&dyn Queue> = self.queues.values().map(AsRef::as_ref).collect();

        queues.sort_by_key(|queue| (queue.stage(), queue.domain()));
        queues.into_iter()
    }

    fn queue_of_mut(&mut self, domain: &str) -> Option<&mut Box<dyn Queue>> {
        self.queues
            .values_mut()
            .find(|queue| queue.domain() == domain)
    }

    fn queue<D>(&self) -> Option<&DomainQueue<D>>
    where
        D: Domain,
    {
        self.queues
            .get(&TypeId::of::<D>())
            .and_then(|queue| queue.as_any().downcast_ref())
    }

    /// `None` only if the queue under `D`'s type id belongs to another
    /// domain, which `queue_mut` itself never stores.
    fn queue_mut<D>(&mut self) -> Option<&mut DomainQueue<D>>
    where
        D: Domain,
    {
        let queue = self
            .queues
            .entry(TypeId::of::<D>())
            .or_insert_with(|| Box::new(DomainQueue::<D>::default()))
            .as_any_mut()
            .downcast_mut();

        if queue.is_none() {
            tracing::error!("the queue of {} holds another domain's changes", D::ID);
        }

        queue
    }
}

impl fmt::Debug for ChangeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.views()).finish()
    }
}
