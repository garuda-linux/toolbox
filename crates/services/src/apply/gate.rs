//! Back pressure on program output. A step that prints faster than the
//! window takes it waits once [`MAX_UNREAD`] bytes are unread, so the
//! reports never pile up in memory: the pipe read pauses and the program
//! blocks on its own output.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use tokio::runtime::Handle;
use tokio::sync::Semaphore;

/// Several frames' worth of what the window feeds its terminal.
const MAX_UNREAD: usize = 1024 * 1024;

#[derive(Clone, Debug)]
pub(super) struct OutputGate(Arc<Semaphore>);

impl Default for OutputGate {
    fn default() -> Self {
        Self(Arc::new(Semaphore::new(MAX_UNREAD)))
    }
}

impl OutputGate {
    /// Waits until `bytes` more fit. A chunk above the limit waits for an
    /// empty queue.
    pub(super) async fn enter(&self, bytes: usize) {
        let permits = u32::try_from(bytes.min(MAX_UNREAD)).unwrap_or(u32::MAX);
        if let Ok(permit) = self.0.acquire_many(permits).await {
            permit.forget();
        }
    }

    /// [`Self::enter`] for the blocking thread of the user steps.
    pub(super) fn enter_blocking(&self, bytes: usize, runtime: &Handle) {
        runtime.block_on(self.enter(bytes));
    }

    /// `bytes` were read. Never frees more than was taken, so reports fed
    /// without the gate leave it as it was.
    pub(super) fn leave(&self, bytes: usize) {
        let taken = MAX_UNREAD.saturating_sub(self.0.available_permits());
        self.0.add_permits(bytes.min(taken));
    }
}
