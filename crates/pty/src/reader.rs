//! The thread that reads the terminal master and hands the output on as
//! text. Waiting for output blocks, so it gets its own thread, which ends
//! when the program and its children have closed the terminal.

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{ErrorKind, Read as _};
use std::sync::mpsc::SyncSender;

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::io::Errno;

const READ_BUFFER: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Chunk {
    /// Raw terminal output, escape sequences included (lossy UTF-8).
    Text(String),
    /// Every holder of the terminal closed it. Nothing follows.
    Closed,
}

/// # Errors
///
/// When the thread cannot be started.
pub fn start(master: File, sender: SyncSender<Chunk>) -> std::io::Result<()> {
    std::thread::Builder::new()
        .name("toolbox-pty".to_owned())
        .spawn(move || pump(master, &sender))
        .map(|_detached| ())
}

fn pump(mut master: File, sender: &SyncSender<Chunk>) {
    let mut buffer = [0_u8; READ_BUFFER];
    let mut pending = Vec::new();

    loop {
        // Linux reports EIO, not end of file, once the last slave closes.
        let read = match master.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                if let Err(failure) = wait_ready(&master, PollFlags::IN, None) {
                    tracing::warn!("cannot wait for terminal output: {failure}");
                    break;
                }

                continue;
            }
            Err(error) => {
                tracing::trace!("terminal closed: {error}");
                break;
            }
        };

        pending.extend_from_slice(buffer.get(..read).unwrap_or_default());
        let text = take_text(&mut pending);

        if !text.is_empty() && sender.send(Chunk::Text(text)).is_err() {
            return;
        }
    }

    if !pending.is_empty() {
        let rest = String::from_utf8_lossy(&pending).into_owned();
        sender.send(Chunk::Text(rest)).ok();
    }

    // Nobody needs the note once the receiving side is gone.
    sender.send(Chunk::Closed).ok();
}

/// Waits until `master` is ready for `events` or `timeout` passed. The
/// master is non-blocking, so a writer that waits for room never holds the
/// thread that also watches for a cancel.
///
/// # Errors
///
/// When the terminal cannot be polled.
pub fn wait_ready(
    master: &File,
    events: PollFlags,
    timeout: Option<&Timespec>,
) -> std::io::Result<()> {
    match poll(&mut [PollFd::new(master, events)], timeout) {
        // A signal only cuts the wait short, the caller tries again.
        Ok(_) | Err(Errno::INTR) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

/// The text decodable from `pending`, which is drained by as much. A
/// character cut off by the end of a read stays for the next one, and so
/// does a partial character behind invalid bytes.
pub fn take_text(pending: &mut Vec<u8>) -> String {
    let mut text = String::new();
    let mut taken = 0;

    for chunk in pending.utf8_chunks() {
        let (valid, invalid) = (chunk.valid(), chunk.invalid());
        text.push_str(valid);
        let end = taken + valid.len() + invalid.len();

        if !invalid.is_empty() {
            if end == pending.len() && is_cut_off(invalid) {
                taken += valid.len();
                break;
            }

            text.push('\u{fffd}');
        }

        taken = end;
    }

    pending.drain(..taken);
    text
}

fn is_cut_off(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_err_and(|error| error.error_len().is_none())
}
