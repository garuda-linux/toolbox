//! A roundtrip to the compositor that gives up. The guest queues ask the
//! compositor before the first frame, and a stalled compositor must not
//! stall the window with them.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use wayland_client::protocol::wl_callback::{self, WlCallback};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};

pub const ROUNDTRIP_LIMIT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Default)]
pub struct Answered(Arc<AtomicBool>);

#[derive(Debug)]
pub enum Stalled {
    TimedOut,
    Wayland(String),
}

impl std::fmt::Display for Stalled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimedOut => write!(f, "no answer within {} s", ROUNDTRIP_LIMIT.as_secs()),
            Self::Wayland(error) => f.write_str(error),
        }
    }
}

/// The body of a guest state's `Dispatch<WlCallback, Answered>`.
pub fn answer(answered: &Answered, event: &wl_callback::Event) {
    if let wl_callback::Event::Done { callback_data: _ } = event {
        answered.0.store(true, Ordering::Release);
    }
}

/// Dispatches `queue` until the compositor answered a `sync`, or
/// [`ROUNDTRIP_LIMIT`] passed.
///
/// # Errors
///
/// [`Stalled::TimedOut`] after the limit, [`Stalled::Wayland`] when the
/// connection fails.
pub fn within<D>(
    connection: &Connection,
    queue: &mut EventQueue<D>,
    state: &mut D,
) -> Result<(), Stalled>
where
    D: Dispatch<WlCallback, Answered> + 'static,
{
    let answered = Answered::default();
    let handle: QueueHandle<D> = queue.handle();
    connection.display().sync(&handle, answered.clone());
    let deadline = Instant::now() + ROUNDTRIP_LIMIT;

    loop {
        queue
            .dispatch_pending(state)
            .map_err(|error| Stalled::Wayland(error.to_string()))?;

        if answered.0.load(Ordering::Acquire) {
            return Ok(());
        }

        connection
            .flush()
            .map_err(|error| Stalled::Wayland(error.to_string()))?;

        let Some(guard) = queue.prepare_read() else {
            continue;
        };

        let left = deadline
            .checked_duration_since(Instant::now())
            .ok_or(Stalled::TimedOut)?;

        let timeout = Timespec::try_from(left).map_err(|_too_long| Stalled::TimedOut)?;
        let fd = guard.connection_fd();
        let mut fds = [PollFd::new(&fd, PollFlags::IN)];
        let ready =
            poll(&mut fds, Some(&timeout)).map_err(|error| Stalled::Wayland(error.to_string()))?;

        if ready == 0 {
            return Err(Stalled::TimedOut);
        }

        guard
            .read()
            .map_err(|error| Stalled::Wayland(error.to_string()))?;
    }
}
