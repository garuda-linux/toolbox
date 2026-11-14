//! A page's click as an edit: its service worked out the edit against the
//! queue it was shown, and the window takes it here in the same frame.

#[cfg(test)]
mod tests;

use toolbox_core::{ChangeSet, Snapshot, domains};

use super::Queue;
use super::edit::QueueEdit;

impl Queue {
    /// Takes `edit` against `snapshot` as the click it came from. Unlike
    /// [`Queue::take`] the guards stay as they were: the page's rules
    /// already met the click, and running every guard again would hold
    /// Apply until each part is read once more.
    pub fn apply_edit(&mut self, edit: &QueueEdit, snapshot: &Snapshot) {
        if edit.is_empty() {
            return;
        }

        let domains = match domains::all() {
            Ok(domains) => domains,
            Err(error) => {
                tracing::error!("the change domains are broken: {error}");
                return;
            }
        };

        let before = self.changes.to_saved();
        for problem in edit.apply(&mut self.changes, &domains, snapshot) {
            tracing::warn!("dropped a change of a page: {problem}");
        }

        if self.changes.to_saved() != before {
            self.save();
        }
    }
}

impl QueueEdit {
    /// What `click` does to a copy of `changes`, and what it answered, so
    /// a page's service works out its edit without the queue itself.
    pub fn of_click<R>(changes: &ChangeSet, click: impl FnOnce(&mut ChangeSet) -> R) -> (Self, R) {
        let mut copy = changes.clone();
        let answer = click(&mut copy);
        (Self::between(changes, &copy), answer)
    }
}
