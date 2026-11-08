//! Ctrl+C on the command line, counted instead of ending the process. In
//! a change the first lets the running step end and skips the rest, the
//! second stops the running step now. The command line waits for the
//! helper either way, so it can say how the change ended and the cleanup
//! of a step is never cut off. `status --watch` ends on the first.
//! SIGTERM and SIGHUP count as Ctrl+C does, so a closed terminal or a
//! `kill` stops a change as cleanly.

#[cfg(test)]
mod tests;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

use tokio::runtime::Handle;
use tokio::signal::unix::{SignalKind, signal};
use toolbox_cli::Exit;
use toolbox_services::apply::ApplyCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Interrupt,
    Terminate,
    HangUp,
}

#[derive(Debug, Default)]
pub struct Presses {
    count: AtomicU8,
    /// `0` until a signal came, else its [`Signal::number`].
    first: AtomicU8,
}

#[derive(Debug)]
pub struct Stopper {
    presses: Arc<Presses>,
    start: u8,
    seen: u8,
}

impl Signal {
    const ALL: [Self; 3] = [Self::Interrupt, Self::Terminate, Self::HangUp];

    /// The code a shell reports for a command this signal ended: 128 and
    /// the signal's number.
    pub const fn exit(self) -> Exit {
        match self {
            Self::Interrupt => Exit::Interrupted,
            Self::Terminate => Exit::Terminated,
            Self::HangUp => Exit::HungUp,
        }
    }

    const fn number(self) -> u8 {
        match self {
            Self::HangUp => 1,
            Self::Interrupt => 2,
            Self::Terminate => 15,
        }
    }

    const fn kind(self) -> SignalKind {
        match self {
            Self::Interrupt => SignalKind::interrupt(),
            Self::Terminate => SignalKind::terminate(),
            Self::HangUp => SignalKind::hangup(),
        }
    }

    fn from_number(number: u8) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|signal| signal.number() == number)
    }
}

impl Presses {
    #[cfg(test)]
    pub fn press(&self) {
        self.signal(Signal::Interrupt);
    }

    pub fn signal(&self, signal: Signal) {
        self.first
            .compare_exchange(0, signal.number(), Ordering::Relaxed, Ordering::Relaxed)
            .ok();

        self.count
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                Some(count.saturating_add(1))
            })
            .ok();
    }

    fn count(&self) -> u8 {
        self.count.load(Ordering::Relaxed)
    }

    fn first(&self) -> Signal {
        Signal::from_number(self.first.load(Ordering::Relaxed)).unwrap_or(Signal::Interrupt)
    }
}

impl Stopper {
    pub fn new(presses: Arc<Presses>) -> Self {
        let start = presses.count();
        Self {
            presses,
            start,
            seen: 0,
        }
    }

    /// The numbers of the presses since the last call, counted from the
    /// start of the run.
    pub fn take(&mut self) -> Vec<u8> {
        let now = self.presses.count().saturating_sub(self.start);
        let new = (self.seen.saturating_add(1)..=now).collect();
        self.seen = now;
        new
    }

    pub fn exit(&self) -> Exit {
        self.presses.first().exit()
    }
}

pub const fn command(press: u8) -> Option<ApplyCommand> {
    match press {
        1 => Some(ApplyCommand::Finish),
        2 => Some(ApplyCommand::Cancel),
        _ => None,
    }
}

pub const fn note(press: u8) -> Option<&'static str> {
    match press {
        1 => Some("stopping after this step, press Ctrl+C again to stop it now"),
        2 => Some("stopping now, the step's cleanup still runs"),
        _ => None,
    }
}

/// The presses of this process. The first call takes Ctrl+C, SIGTERM and
/// SIGHUP over for good: their default would end the command line in the
/// middle of a change.
pub fn listen(runtime: &Handle) -> Arc<Presses> {
    static PRESSES: OnceLock<Arc<Presses>> = OnceLock::new();

    let presses = PRESSES.get_or_init(|| {
        let presses = Arc::new(Presses::default());

        // Registered here, not in the task, so a Ctrl+C right after the
        // question is already caught.
        let _runtime = runtime.enter();
        for kind in Signal::ALL {
            match signal(kind.kind()) {
                Ok(mut signals) => {
                    let counted = Arc::clone(&presses);
                    drop(runtime.spawn(async move {
                        while signals.recv().await.is_some() {
                            counted.signal(kind);
                        }
                    }));
                }
                Err(error) => tracing::warn!("{kind:?} will end the command at once: {error}"),
            }
        }

        presses
    });

    Arc::clone(presses)
}
