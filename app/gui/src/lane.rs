//! A page's reads on the runtime: each wakes the frame with its answer,
//! and of two reads of one kind only the newest answer lands, so an older
//! read that ends last cannot undo a newer one.

#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use crossbeam_channel::Receiver;
use tokio::runtime::Handle;
use tokio::task::{JoinError, JoinHandle};

use crate::spawn::spawn_logged;
use crate::waking::{self, Wake, Waking};

/// What starts one read of an [`Asked`].
type Start<T> = Box<dyn Fn(&Lane<T>) + Send>;

struct Ticket {
    kind: &'static str,
    number: u64,
}

/// Only the UI thread asks and takes, so the newest tickets need no lock.
pub struct Lane<T> {
    runtime: Handle,
    sender: Waking<(Ticket, T)>,
    receiver: Receiver<(Ticket, T)>,
    newest: RefCell<HashMap<&'static str, u64>>,
}

/// One read of a page's own at a time. Asks that come while it runs
/// become one more read once it landed.
#[derive(Debug, Default)]
pub struct OneRead {
    running: bool,
    again: bool,
}

/// A page's read of one kind that it asks for again and again, as on
/// opening the page and on Refresh: a [`Lane`] with a [`OneRead`], and
/// what starts the read. What the page shows meanwhile stays the page's.
pub struct Asked<T> {
    lane: Lane<T>,
    reads: OneRead,
    start: Start<T>,
}

impl<T> fmt::Debug for Lane<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Lane").finish_non_exhaustive()
    }
}

impl<T> fmt::Debug for Asked<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Asked")
            .field("reads", &self.reads)
            .finish_non_exhaustive()
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
            newest: RefCell::default(),
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
        let newest = self.newest.borrow();

        self.receiver
            .try_iter()
            .filter(|(ticket, _)| newest.get(ticket.kind) == Some(&ticket.number))
            .map(|(_, answer)| answer)
            .collect()
    }

    fn ticket(&self, kind: &'static str) -> Ticket {
        let mut newest = self.newest.borrow_mut();
        let number = newest.get(kind).map_or(0, |last| last + 1);
        newest.insert(kind, number);

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

impl<T: Send + 'static> Asked<T> {
    /// `start` asks the lane for one read, of one kind.
    #[must_use = "nothing is read until the page asks"]
    pub fn new(runtime: Handle, wake: Wake, start: impl Fn(&Lane<T>) + Send + 'static) -> Self {
        Self {
            lane: Lane::new(runtime, wake),
            reads: OneRead::default(),
            start: Box::new(start),
        }
    }

    /// Starts a read. While one runs it is asked for once that landed.
    /// Returns whether one started, so the page shows it loads.
    pub fn read(&mut self) -> bool {
        let started = self.reads.start();
        if started {
            (self.start)(&self.lane);
        }

        started
    }

    /// Whether a read runs now, for a Read again that spins meanwhile.
    pub const fn running(&self) -> bool {
        self.reads.running
    }

    /// The answer that landed since the last frame, if any. A read asked
    /// meanwhile starts then.
    pub fn take(&mut self) -> Option<T> {
        let answer = self.lane.landed().pop()?;
        if self.reads.landed() {
            self.read();
        }

        Some(answer)
    }
}
