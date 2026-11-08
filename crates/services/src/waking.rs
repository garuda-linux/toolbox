//! A channel that wakes its reader with every value, so an answer from
//! another thread shows without waiting for input.
//!
//! What waking means is the reader's: the window repaints, the command
//! line has nothing to do.

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};

#[derive(Clone)]
pub struct Wake(Arc<dyn Fn() + Send + Sync>);

pub struct Waking<T> {
    sender: Sender<T>,
    wake: Wake,
}

impl Wake {
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self(Arc::new(wake))
    }

    /// For a reader without a window, such as the command line.
    #[must_use = "the wake is given to a service"]
    pub fn none() -> Self {
        Self::new(|| {})
    }

    pub fn wake(&self) {
        (self.0)();
    }
}

impl fmt::Debug for Wake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wake").finish_non_exhaustive()
    }
}

impl<T> Clone for Waking<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            wake: self.wake.clone(),
        }
    }
}

impl<T> fmt::Debug for Waking<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Waking").finish_non_exhaustive()
    }
}

impl<T> Waking<T> {
    pub fn send(&self, value: T) {
        // An error only means the receiver is gone: the app or its page
        // closed.
        self.sender.send(value).ok();
        self.wake.wake();
    }
}

/// Unbounded: the values are small, and a sender must not stall on a
/// reader that is late.
#[must_use = "both ends are needed"]
pub fn channel<T>(wake: Wake) -> (Waking<T>, Receiver<T>) {
    let (sender, receiver) = crossbeam_channel::unbounded();
    (Waking { sender, wake }, receiver)
}
