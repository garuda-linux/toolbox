//! The queued changes. They are kept in `toolbox-config`'s
//! `changes.json`, so a queue survives a restart.

mod answer;
mod applying;
pub mod edit;
#[cfg(test)]
mod tests;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crossbeam_channel::Receiver;
use tokio::runtime::Handle;
use tokio::sync::watch;
use toolbox_config::write_atomic;
use toolbox_core::features::Unchecked;
use toolbox_core::{ChangeSet, ChangeView, Domains, SavedChanges, Snapshot, domains};
use toolbox_plan::{Plan, StepId};

use self::applying::Applying;
use self::edit::QueueEdit;
use crate::waking::{self, Wake};

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
    /// The newest state for the writer task. `None` keeps the queue in
    /// memory, as tests do.
    writer: Option<watch::Sender<SavedChanges>>,
    /// The writer task's failed writes, for the window to tell of.
    not_kept: Option<Receiver<NotKept>>,
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
    /// A queue that is never written, for tests.
    pub fn in_memory(changes: ChangeSet) -> Self {
        Self {
            unchecked: unchecked(&changes),
            changes,
            writer: None,
            not_kept: None,
            revision: 0,
            applying: Applying::None,
        }
    }

    /// A queue written to `path` after every change. One task writes, so
    /// writes keep their order, and a burst of changes is written once.
    /// A failed write wakes the window through `wake`.
    pub fn with_file(changes: ChangeSet, path: PathBuf, runtime: &Handle, wake: Wake) -> Self {
        let (writer, mut newest) = watch::channel(changes.to_saved());
        newest.mark_unchanged();
        let (failed, not_kept) = waking::channel(wake);

        drop(runtime.spawn(async move {
            while newest.changed().await.is_ok() {
                let saved = newest.borrow_and_update().clone();
                let path = path.clone();
                match tokio::task::spawn_blocking(move || write(path, &saved)).await {
                    Ok(Ok(())) => {}
                    Ok(Err(failure)) => {
                        tracing::warn!("{failure}");
                        failed.send(failure);
                    }
                    Err(_stopped) => {
                        tracing::warn!("the queue writer stopped");
                        return;
                    }
                }
            }
        }));

        Self {
            unchecked: unchecked(&changes),
            changes,
            writer: Some(writer),
            not_kept: Some(not_kept),
            revision: 0,
            applying: Applying::None,
        }
    }

    /// The newest failed write since the last call, so the window tells
    /// of a burst of failures once.
    pub fn not_kept(&self) -> Option<NotKept> {
        self.not_kept.as_ref()?.try_iter().last()
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

    /// The steps that carry out the queue, in the planner's order.
    #[must_use = "the plan does nothing until applied"]
    pub fn plan(&self) -> Plan {
        toolbox_core::plan(&self.changes)
    }

    /// Apply sent `plan`, made of this queue.
    pub fn sent(&mut self, plan: &Plan) {
        self.applying = Applying::Sent(plan.clone());
    }

    pub fn plan_started(&mut self) {
        self.applying = std::mem::take(&mut self.applying).started();
    }

    pub fn plan_refused(&mut self) {
        self.applying = std::mem::take(&mut self.applying).refused();
    }

    /// After the queue's plan: one-off changes (DKMS builds, account
    /// changes, maintenance) whose steps `succeeded` names are done and
    /// leave the queue. The end of another plan changes nothing. Desired
    /// state stays until a new snapshot shows the system fulfils it.
    pub fn plan_finished(&mut self, succeeded: impl Fn(StepId) -> bool) {
        if let Some(done) = self.applying.finished(succeeded) {
            self.changes.drop_done_one_offs(&done);
            self.save();
        }
    }

    /// Changes with every change to the queue.
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    fn save(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        if let Some(writer) = &self.writer {
            writer.send_replace(self.changes.to_saved());
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

/// The queue saved at `path`, compared with `snapshot`. A missing or
/// broken file is an empty queue: a lost queue is annoying, an app that
/// cannot start is worse.
pub fn read(path: &Path, snapshot: &Snapshot) -> ChangeSet {
    let mut changes = ChangeSet::default();
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return changes,
        Err(error) => {
            tracing::warn!(
                "cannot read the queued changes in {}: {error}",
                path.display()
            );
            return changes;
        }
    };

    let saved = match serde_json::from_str::<SavedChanges>(&text) {
        Ok(saved) => saved,
        Err(error) => {
            tracing::warn!("cannot understand {}: {error}", path.display());
            return changes;
        }
    };

    match domains::all() {
        Ok(domains) => {
            for problem in domains.restore(&mut changes, &saved, snapshot) {
                tracing::warn!("dropped a queued change: {problem}");
            }
        }
        Err(error) => tracing::error!("the change domains are broken: {error}"),
    }

    changes
}

/// Keeps `saved` at `path` atomically. An empty queue removes the file
/// instead of keeping an empty one. Blocks on two file syncs.
///
/// # Errors
///
/// When the file cannot be written or removed.
pub fn store(path: &Path, saved: &SavedChanges) -> std::io::Result<()> {
    if saved.changes.is_empty() {
        match std::fs::remove_file(path) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(error),
            Ok(()) | Err(_) => Ok(()),
        }
    } else {
        let bytes = serde_json::to_vec_pretty(saved).map_err(std::io::Error::other)?;
        write_atomic(path, &bytes)
    }
}

fn write(path: PathBuf, saved: &SavedChanges) -> Result<(), NotKept> {
    store(&path, saved).map_err(|source| NotKept { path, source })
}
