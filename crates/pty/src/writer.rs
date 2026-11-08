//! The thread that types input into the terminal. A program that stops
//! reading fills the terminal's input buffer, and a write then waits for
//! room. That wait happens here, so the caller's thread stays free to
//! read output, cancel and see the exit.

use std::fs::File;
use std::io::{self, Write as _};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};

use rustix::event::{PollFlags, Timespec};

use crate::reader;

/// Inputs that may wait for the program. The app sends a paste in parts
/// of at most 64 KiB, so this holds a few MiB. More waiting input means
/// the program does not read, and is refused instead of piling up.
pub const INPUT_QUEUE: usize = 32;

/// How long one wait for room in the terminal lasts before the thread
/// looks at the abandon flag again.
const WRITE_WAIT: Timespec = Timespec {
    tv_sec: 0,
    tv_nsec: 50_000_000,
};

/// The caller's end of the writer thread. Dropping it abandons whatever
/// input still waits.
#[derive(Debug)]
pub struct Writer {
    queue: SyncSender<Vec<u8>>,
    abandoned: Arc<AtomicBool>,
}

impl Writer {
    /// # Errors
    ///
    /// When the thread cannot be started.
    pub fn start(master: File) -> io::Result<Self> {
        let (queue, inputs) = mpsc::sync_channel(INPUT_QUEUE);
        let abandoned = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&abandoned);

        std::thread::Builder::new()
            .name("toolbox-pty-input".to_owned())
            .spawn(move || pump(&master, &inputs, &flag))
            .map(|_detached| Self { queue, abandoned })
    }

    /// # Errors
    ///
    /// `WouldBlock` when [`INPUT_QUEUE`] inputs still wait, `BrokenPipe`
    /// once the writer gave up (abandoned or the terminal closed).
    pub fn send(&self, data: &[u8]) -> io::Result<()> {
        if self.abandoned.load(Ordering::Relaxed) {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "the input was abandoned",
            ));
        }

        match self.queue.try_send(data.to_vec()) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "the program does not read its input",
            )),
            Err(TrySendError::Disconnected(_)) => Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "the terminal takes no more input",
            )),
        }
    }

    /// Drops the input that waits and what is still to come. After a
    /// cancel nothing typed is meant for the program any more.
    pub fn abandon(&self) {
        self.abandoned.store(true, Ordering::Relaxed);
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        self.abandon();
    }
}

fn pump(master: &File, inputs: &Receiver<Vec<u8>>, abandoned: &AtomicBool) {
    while let Ok(data) = inputs.recv() {
        if let Err(error) = write_all(master, &data, abandoned) {
            tracing::debug!("input to the terminal stopped: {error}");
            return;
        }
    }
}

/// Writes `data` whole, waiting for room in steps so an abandon ends it.
fn write_all(mut master: &File, data: &[u8], abandoned: &AtomicBool) -> io::Result<()> {
    let mut rest = data;

    while !rest.is_empty() {
        if abandoned.load(Ordering::Relaxed) {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "the input was abandoned",
            ));
        }

        match master.write(rest) {
            Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
            Ok(written) => rest = rest.get(written..).unwrap_or_default(),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                reader::wait_ready(master, PollFlags::OUT, Some(&WRITE_WAIT))?;
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }

    Ok(())
}
