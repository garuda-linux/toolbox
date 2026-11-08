//! Stopping a program in steps: a polite interrupt first, harder signals
//! when it does not react. The caller drives it with the current time, so
//! no timer thread is needed and tests pass time in.

#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

/// How long a program gets after each signal before the next, harder one.
/// pacman needs a moment after SIGINT to release its database lock.
pub const CANCEL_GRACE: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    /// SIGINT, what Ctrl+C sends. pacman and paru clean up on it.
    Interrupt,
    Terminate,
    /// SIGKILL, for a program that ignored both.
    Kill,
}

impl Signal {
    pub(crate) const fn to_rustix(self) -> rustix::process::Signal {
        match self {
            Self::Interrupt => rustix::process::Signal::INT,
            Self::Terminate => rustix::process::Signal::TERM,
            Self::Kill => rustix::process::Signal::KILL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Idle,
    Interrupted(Instant),
    Terminated(Instant),
    Killed,
}

/// Where a cancel stands: which signal went out last and when.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cancellation {
    stage: Stage,
    grace: Duration,
}

impl Cancellation {
    /// A cancellation that waits `grace` between signals.
    #[must_use = "a cancellation does nothing until started"]
    pub const fn new(grace: Duration) -> Self {
        Self {
            stage: Stage::Idle,
            grace,
        }
    }

    pub const fn is_requested(&self) -> bool {
        !matches!(self.stage, Stage::Idle)
    }

    /// Starts the sequence at `now`. Returns the signal to send, or `None`
    /// when a cancel is already under way.
    pub const fn start(&mut self, now: Instant) -> Option<Signal> {
        if self.is_requested() {
            return None;
        }

        self.stage = Stage::Interrupted(now);
        Some(Signal::Interrupt)
    }

    /// The signal that is due at `now`, if any. Each one is returned once.
    pub fn poll(&mut self, now: Instant) -> Option<Signal> {
        let (since, next, signal) = match self.stage {
            Stage::Interrupted(since) => (since, Stage::Terminated(now), Signal::Terminate),
            Stage::Terminated(since) => (since, Stage::Killed, Signal::Kill),
            Stage::Idle | Stage::Killed => return None,
        };

        if now.saturating_duration_since(since) < self.grace {
            return None;
        }

        self.stage = next;
        Some(signal)
    }
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new(CANCEL_GRACE)
    }
}
