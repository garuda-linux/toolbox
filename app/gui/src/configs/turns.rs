//! The order of the work on the folder of private copies. Each job runs
//! on a blocking thread of its own, so without turns the emptying at the
//! start could run after the first edit made its copy, and the removal of
//! one edit's copy after the next edit of the same file made a new one.

#[cfg(test)]
mod tests;

use parking_lot::Mutex;
use tokio::sync::oneshot;

/// Hands out turns in the order the frame asks for them.
#[derive(Debug, Default)]
pub struct Turns {
    /// Closes when the newest turn ended.
    last: Mutex<Option<oneshot::Receiver<()>>>,
}

/// One job's place in the line. Its end, dropped or done, is what lets
/// the next turn go, so a job that fails or panics never holds the line.
#[derive(Debug)]
pub struct Turn {
    before: Option<oneshot::Receiver<()>>,
    _ended: oneshot::Sender<()>,
}

impl Turns {
    /// Taken on the frame, when the job is asked for, since the threads
    /// may start in any order.
    pub fn take(&self) -> Turn {
        let (ended, next) = oneshot::channel();
        let before = self.last.lock().replace(next);

        Turn {
            before,
            _ended: ended,
        }
    }
}

impl Turn {
    /// Blocks until every earlier turn ended. Only on a blocking thread.
    pub fn wait(&mut self) {
        if let Some(before) = self.before.take() {
            // An error only means the earlier turn ended.
            drop(before.blocking_recv());
        }
    }
}
