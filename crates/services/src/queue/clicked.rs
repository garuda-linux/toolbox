//! A page's click: its service worked it out on a copy of the queue it
//! was shown, and the window swaps the copy in within the same frame.

#[cfg(test)]
mod tests;

use toolbox_core::ChangeSet;

use super::Queue;

impl Queue {
    /// Takes `changes`, a copy of this queue a click changed. Unlike
    /// [`Queue::take`] the guards stay as they were: the page's rules
    /// already met the click, and running every guard again would hold
    /// Apply until each part is read once more.
    pub fn take_click(&mut self, changes: ChangeSet) {
        let before = self.changes.to_saved();
        self.changes = changes;

        self.save_if_changed(&before);
    }
}

/// What `click` does to a copy of `changes`, and what it answered, so a
/// page's service works out its click without the queue itself.
pub fn on_copy<R>(changes: &ChangeSet, click: impl FnOnce(&mut ChangeSet) -> R) -> (ChangeSet, R) {
    let mut copy = changes.clone();
    let answer = click(&mut copy);
    (copy, answer)
}
