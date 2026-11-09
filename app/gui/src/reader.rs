//! Runs a slow read of the system off the UI thread and hands the newest
//! result to the frame.
//!
//! It reads at start, on every request and, with a period, again once the
//! period has passed since the last read. Reads block, so they run on
//! tokio's blocking threads. Requests that pile up while a read runs
//! become one more read.

#[cfg(test)]
mod tests;

use std::convert::Infallible;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use tokio::runtime::Handle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::watch;
use tokio::time::Instant;
use toolbox_gui_kit::Loadable;

use crate::spawn::spawn_logged;
use crate::waking::Wake;

#[derive(Debug)]
pub struct ReaderHandle<T> {
    requests: UnboundedSender<Request>,
    reading: watch::Receiver<Loadable<T>>,
}

/// A reader and its last reading, as the app keeps them. Without a reader
/// (tests that need none) nothing is ever loaded.
#[derive(Debug)]
pub struct Watched<T> {
    handle: Option<ReaderHandle<T>>,
    reading: Loadable<T>,
}

#[derive(Debug)]
enum Request {
    Read,
    Every(Option<Duration>),
}

impl<T> ReaderHandle<T> {
    pub fn refresh(&self) {
        self.send(Request::Read);
    }

    /// Reads again every `period` after the last read. `None` stops that.
    pub fn every(&self, period: Option<Duration>) {
        self.send(Request::Every(period));
    }

    fn send(&self, request: Request) {
        if self.requests.send(request).is_err() {
            tracing::debug!("a reader is gone");
        }
    }

    pub fn changed(&mut self) -> Option<Loadable<T>> {
        fresh(&mut self.reading)
    }
}

impl<T> Watched<T> {
    pub fn new(handle: Option<ReaderHandle<T>>) -> Self {
        Self {
            handle,
            reading: Loadable::default(),
        }
    }

    pub fn take(&mut self) {
        if let Some(reading) = self.handle.as_mut().and_then(ReaderHandle::changed) {
            self.reading = reading;
        }
    }

    pub fn refresh(&self) {
        if let Some(handle) = &self.handle {
            handle.refresh();
        }
    }

    /// Reads again every `period`, or only when asked with `None`.
    pub fn every(&self, period: Option<Duration>) {
        if let Some(handle) = &self.handle {
            handle.every(period);
        }
    }

    pub const fn reading(&self) -> &Loadable<T> {
        &self.reading
    }

    /// The reading while a reader feeds it, `None` without one.
    pub const fn fed(&self) -> Option<&Loadable<T>> {
        match self.handle {
            Some(_) => Some(&self.reading),
            None => None,
        }
    }

    pub fn latest(&self) -> Option<&T> {
        self.reading.get()
    }
}

/// The value `receiver` holds, once per change. An error means the task
/// that sends ended, and the last value stays.
pub fn fresh<T>(receiver: &mut watch::Receiver<T>) -> Option<T>
where
    T: Clone,
{
    receiver
        .has_changed()
        .unwrap_or(false)
        .then(|| receiver.borrow_and_update().clone())
}

/// Starts the reader and its first read. `what` names it in the log.
#[must_use = "the handle is the only way to the reader"]
pub fn spawn<T, R>(runtime: &Handle, wake: Wake, what: &'static str, read: R) -> ReaderHandle<T>
where
    T: Send + Sync + 'static,
    R: Fn() -> T + Send + Sync + 'static,
{
    spawn_fallible(runtime, wake, what, move || Ok::<T, Infallible>(read()))
}

/// [`spawn`] for a read that can fail. The reading then keeps the last
/// value beside the error's text, which the page shows.
#[must_use = "the handle is the only way to the reader"]
pub fn spawn_fallible<T, E, R>(
    runtime: &Handle,
    wake: Wake,
    what: &'static str,
    read: R,
) -> ReaderHandle<T>
where
    T: Send + Sync + 'static,
    E: Display + Send + 'static,
    R: Fn() -> Result<T, E> + Send + Sync + 'static,
{
    let (requests, queue) = unbounded_channel();
    let (sender, reading) = watch::channel(Loadable::default());
    let handle = ReaderHandle { requests, reading };

    handle.refresh();
    spawn_logged(
        runtime,
        what,
        serve(queue, Arc::new(read), sender, wake, what),
    );
    handle
}

async fn serve<T, E, R>(
    mut requests: UnboundedReceiver<Request>,
    read: Arc<R>,
    outlet: watch::Sender<Loadable<T>>,
    wake: Wake,
    what: &'static str,
) where
    T: Send + Sync + 'static,
    E: Display + Send + 'static,
    R: Fn() -> Result<T, E> + Send + Sync + 'static,
{
    let mut period = None;
    let mut next: Option<Instant> = None;

    loop {
        let due = tokio::select! {
            request = requests.recv() => match request {
                None => return,
                Some(Request::Read) => true,
                Some(Request::Every(new_period)) => {
                    period = new_period;
                    next = period.map(|every| Instant::now() + every);
                    false
                }
            },
            () = sleep_until(next) => true,
        };

        if !due {
            continue;
        }

        while let Ok(request) = requests.try_recv() {
            if let Request::Every(new_period) = request {
                period = new_period;
            }
        }

        outlet.send_modify(Loadable::start_loading);
        wake.wake();

        let run = Arc::clone(&read);

        // Its close line says how long the read took.
        let span = tracing::debug_span!("read", what);
        let result = match tokio::task::spawn_blocking(move || span.in_scope(|| run())).await {
            Ok(answer) => answer.map_err(|error| error.to_string()),
            Err(error) => {
                tracing::error!("reading {what} stopped: {error}");
                Err(error.to_string())
            }
        };

        outlet.send_modify(|reading| reading.finish(result));
        wake.wake();
        next = period.map(|every| Instant::now() + every);
    }
}

/// Never wakes without a deadline.
async fn sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}
