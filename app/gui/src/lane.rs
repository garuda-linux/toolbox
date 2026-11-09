//! A page's reads on the runtime: each wakes the frame with its answer,
//! and of two reads of one kind only the newest answer lands, so an older
//! read that ends last cannot undo a newer one.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crossbeam_channel::Receiver;
use parking_lot::Mutex;
use tokio::runtime::Handle;
use tokio::task::{JoinError, JoinHandle};

use crate::spawn::spawn_logged;
use crate::waking::{self, Wake, Waking};

struct Ticket {
    kind: &'static str,
    number: u64,
}

pub struct Lane<T> {
    runtime: Handle,
    sender: Waking<(Ticket, T)>,
    receiver: Receiver<(Ticket, T)>,
    newest: Arc<Mutex<HashMap<&'static str, u64>>>,
}

/// One read of a page's own at a time. Asks that come while it runs
/// become one more read once it landed.
#[derive(Debug, Default)]
pub struct OneRead {
    running: bool,
    again: bool,
}

impl<T> fmt::Debug for Lane<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lane").finish_non_exhaustive()
    }
}

impl<T: Send + 'static> Lane<T> {
    #[must_use = "the lane runs nothing until a read is asked of it"]
    pub fn new(runtime: Handle, wake: Wake) -> Self {
        let (sender, receiver) = waking::channel(wake);

        Self {
            runtime,
            sender,
            receiver,
            newest: Arc::default(),
        }
    }

    /// Runs `work` on a blocking thread and lands what `answer` makes of
    /// its result. A panic arrives as the [`JoinError`] that carries its
    /// message.
    pub fn blocking<R, W, A>(&self, kind: &'static str, work: W, answer: A)
    where
        R: Send + 'static,
        W: FnOnce() -> R + Send + 'static,
        A: FnOnce(Result<R, JoinError>) -> T + Send + 'static,
    {
        let task = self.runtime.spawn_blocking(work);
        self.land(kind, task, answer);
    }

    /// Lands what `answer` makes of the output of `future`. A panic arrives
    /// as its [`JoinError`] too.
    pub fn future<F, A>(&self, kind: &'static str, future: F, answer: A)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
        A: FnOnce(Result<F::Output, JoinError>) -> T + Send + 'static,
    {
        let task = self.runtime.spawn(future);
        self.land(kind, task, answer);
    }

    /// The task runs on its own, so a panic in it ends only it and the
    /// ticket still lands.
    fn land<R, A>(&self, kind: &'static str, task: JoinHandle<R>, answer: A)
    where
        R: Send + 'static,
        A: FnOnce(Result<R, JoinError>) -> T + Send + 'static,
    {
        let ticket = self.ticket(kind);
        let sender = self.sender.clone();

        spawn_logged(&self.runtime, "lane answer", async move {
            sender.send((ticket, answer(task.await)));
        });
    }

    /// The answers that arrived, without those a newer read of their kind
    /// has made stale.
    pub fn landed(&self) -> Vec<T> {
        let newest = self.newest.lock();

        self.receiver
            .try_iter()
            .filter(|(ticket, _)| newest.get(ticket.kind) == Some(&ticket.number))
            .map(|(_, answer)| answer)
            .collect()
    }

    fn ticket(&self, kind: &'static str) -> Ticket {
        let number = {
            let mut newest = self.newest.lock();
            let number = newest.get(kind).map_or(0, |last| last + 1);
            newest.insert(kind, number);
            number
        };

        Ticket { kind, number }
    }
}

impl OneRead {
    /// Whether a read may start now. While one runs the ask waits.
    pub const fn start(&mut self) -> bool {
        if self.running {
            self.again = true;
            return false;
        }

        self.running = true;
        true
    }

    /// The read landed. Returns whether another one was asked meanwhile.
    pub fn landed(&mut self) -> bool {
        self.running = false;
        std::mem::take(&mut self.again)
    }
}
