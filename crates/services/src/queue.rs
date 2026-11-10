//! The queued changes. They are kept in `toolbox-config`'s
//! `changes.json`, so a queue survives a restart.

mod answer;
mod applying;
pub mod edit;
mod file;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use tokio::runtime::Handle;
use toolbox_core::features::{Guard, Refusal, Unchecked};
use toolbox_core::{ChangeSet, ChangeView, Domains, Owners, Snapshot};
use toolbox_plan::{Plan, PlannedAction, StepId};

use self::applying::Applying;
use self::edit::QueueEdit;
use self::file::Writer;
pub use self::file::{Restored, keep_aside, read, store};
use crate::waking::Wake;

/// The queue file was not written. The queue stays in memory, so only a
/// restart loses it.
#[derive(Debug, thiserror::Error)]
#[error("cannot keep the queued changes in {}: {source}", path.display())]
pub struct NotKept {
    pub path: PathBuf,
    #[source]
    pub source: std::io::Error,
}

#[derive(Debug)]
pub struct Queue {
    changes: ChangeSet,
    /// The task that writes the file. `None` keeps the queue in memory,
    /// as tests do.
    writer: Option<Writer>,
    /// The pages' guards the queue has not met yet. The queue restored
    /// from `changes.json` came before the first snapshot, so each guard
    /// runs once, with the first snapshot that holds what it checks.
    unchecked: Unchecked,
    /// Counts the changes to the queue, so pages tell a new queue from
    /// the one their rows were built from without comparing it.
    revision: u64,
    applying: Applying,
}

impl Queue {
    /// A queue that is never written, for tests: changes the user queued
    /// in this session, which met every guard when they were queued.
    pub fn in_memory(changes: ChangeSet) -> Self {
        Self {
            unchecked: Unchecked::default(),
            changes,
            writer: None,
            revision: 0,
            applying: Applying::None,
        }
    }

    /// [`Queue::in_memory`] as a restored queue, whose guards still wait.
    pub fn restored_in_memory(changes: ChangeSet) -> Self {
        Self {
            unchecked: unchecked(&changes),
            ..Self::in_memory(changes)
        }
    }

    /// A queue written to `path` after every change. One task writes, so
    /// writes keep their order, and a burst of changes is written once.
    /// A failed write wakes the window through `wake`.
    pub fn with_file(
        changes: ChangeSet,
        path: PathBuf,
        runtime: &Handle,
        wake: Wake,
        restored: Restored,
    ) -> Self {
        let writer = Writer::spawn(changes.to_saved(), path, runtime, wake, restored);

        Self {
            unchecked: unchecked(&changes),
            changes,
            writer: Some(writer),
            revision: 0,
            applying: Applying::None,
        }
    }

    /// The newest failed write since the last call, so the window tells
    /// of a burst of failures once. A writer task that stopped is started
    /// once more first.
    pub fn not_kept(&mut self) -> Option<NotKept> {
        self.writer.as_mut()?.failure()
    }

    /// Writes the queue on the calling thread, for the window's close.
    ///
    /// # Errors
    ///
    /// [`NotKept`] when the file cannot be written.
    pub fn flush(&self) -> Result<(), NotKept> {
        self.writer
            .as_ref()
            .map_or(Ok(()), |writer| writer.flush(&self.changes.to_saved()))
    }

    /// Drops what `snapshot` shows the system already fulfils (an apply
    /// that worked, or a change made outside the app), and what a page's
    /// guard refuses once the snapshot holds what it checks.
    pub fn meet(&mut self, snapshot: &Snapshot) {
        let before = self.changes.to_saved();
        self.changes.reconcile(snapshot);
        self.unchecked.recheck(&mut self.changes, snapshot);

        if self.changes.to_saved() != before {
            self.save();
        }
    }

    /// Lets feature logic change several wishes against `snapshot` at
    /// once. The file is written once, and only when the queue changed.
    pub fn edit<R>(
        &mut self,
        snapshot: &Snapshot,
        edit: impl FnOnce(&mut ChangeSet, &Snapshot) -> R,
    ) -> R {
        let before = self.changes.to_saved();
        let result = edit(&mut self.changes, snapshot);

        if self.changes.to_saved() != before {
            self.save();
        }

        result
    }

    pub const fn changes(&self) -> &ChangeSet {
        &self.changes
    }

    pub fn views(&self) -> Vec<ChangeView> {
        self.changes.views()
    }

    /// Takes an edit from a command as the pages' clicks: each wish
    /// against `snapshot`, then every page's guard once more.
    pub fn take(&mut self, edit: &QueueEdit, domains: &Domains, snapshot: &Snapshot) {
        let before = self.changes.to_saved();
        for problem in edit.apply(&mut self.changes, domains, snapshot) {
            tracing::warn!("dropped a change from the command line: {problem}");
        }

        self.unchecked = Unchecked::all();
        self.unchecked.recheck(&mut self.changes, snapshot);

        if self.changes.to_saved() != before {
            self.save();
        }
    }

    pub fn remove(&mut self, domain: &str, key: &str) {
        if self.changes.remove_key(domain, key) {
            self.save();
        }
    }

    pub fn clear(&mut self) {
        self.changes.clear();
        self.save();
    }

    /// # Errors
    ///
    /// While a page's guard has not checked a restored change yet.
    pub fn ready(&self) -> Result<(), Refusal> {
        self.unchecked.ready(&self.changes)
    }

    /// The guards that hold a restored change back until the snapshot
    /// holds their part, so the window asks for the parts a page reads.
    pub fn holding(&self) -> impl Iterator<Item = Guard> + '_ {
        self.unchecked.holding(&self.changes)
    }

    /// The steps that carry out the queue, in the planner's order.
    ///
    /// # Errors
    ///
    /// While a page's guard has not checked a restored change yet: the
    /// plan could run what that page refuses.
    pub fn plan(&self) -> Result<Plan, Refusal> {
        self.ready()?;
        Ok(toolbox_core::plan(&self.changes))
    }

    /// Apply sent `plan`, made of this queue. A plan that is not this
    /// queue's tells none of its changes how it went.
    pub fn sent(&mut self, plan: &Plan) {
        let (own, owners) = toolbox_core::planned(&self.changes);
        let owners = if own == *plan {
            owners
        } else {
            tracing::warn!("the sent plan is not the queue's, its end marks no change");
            Owners::default()
        };

        self.applying = Applying::Sent(plan.clone(), owners);
    }

    pub fn plan_started(&mut self) {
        self.applying = std::mem::take(&mut self.applying).started();
    }

    pub fn plan_refused(&mut self) {
        self.applying = std::mem::take(&mut self.applying).refused();
    }

    /// After the queue's plan: each change learns from `ended_well` (`None`
    /// in a dry run) how its steps went. One-off changes (DKMS builds,
    /// accounts, maintenance) whose steps `succeeded` names leave the queue.
    /// The end of another plan changes nothing. Desired state stays until a
    /// snapshot shows the system fulfils it and its plan finished.
    pub fn plan_finished(
        &mut self,
        succeeded: impl Fn(StepId) -> bool,
        ended_well: Option<&dyn Fn(StepId) -> bool>,
    ) {
        let Some((plan, owners)) = self.applying.finished() else {
            return;
        };

        if let Some(ended_well) = ended_well {
            self.changes.finished(&owners, ended_well);
        }

        let done: Vec<PlannedAction> = plan
            .steps()
            .iter()
            .filter(|step| succeeded(step.id))
            .map(|step| step.action.clone())
            .collect();

        self.changes.drop_done_one_offs(&done);
        self.save();
    }

    /// Changes with every change to the queue.
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    fn save(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        if let Some(writer) = &self.writer {
            writer.send(self.changes.to_saved());
        }
    }
}

/// Every guard waits for a queue that came in before the system was read.
fn unchecked(changes: &ChangeSet) -> Unchecked {
    if changes.is_empty() {
        Unchecked::default()
    } else {
        Unchecked::all()
    }
}
