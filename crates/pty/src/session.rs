//! The program leads its own session, so its process group holds every child
//! it starts (pacman's hooks) and a cancel reaches them all.

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io;
use std::os::unix::process::ExitStatusExt as _;
use std::process::{Child, ExitStatus};
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

use rustix::process::{Pid, WaitId, WaitIdOptions, kill_process_group, waitid};
use rustix::termios::tcsetwinsize;

use crate::cancel::Cancellation;
use crate::size::winsize;
use crate::writer::Writer;
use crate::{Chunk, PtyCommand, Signal, TerminalSize, clamped, reader, spawn};

/// Bounded, so a caller that falls behind blocks the reader and the program
/// waits on the terminal instead of memory filling up.
const OUTPUT_QUEUE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Code(i32),
    Signal(i32),
}

impl From<ExitStatus> for Exit {
    /// Neither a code nor a signal means the program never ran to an exit, so it
    /// counts as a failed code.
    fn from(status: ExitStatus) -> Self {
        match (status.code(), status.signal()) {
            (Some(code), _) => Self::Code(code),
            (None, Some(signal)) => Self::Signal(signal),
            (None, None) => Self::Code(1),
        }
    }
}

/// Dropping a session that still runs kills its process group.
#[derive(Debug)]
pub struct Session {
    child: Child,
    master: File,
    output: Receiver<Chunk>,
    /// Absent only while [`Self::spawn`] starts it.
    writer: Option<Writer>,
    cancellation: Cancellation,
    exit: Option<Exit>,
}

impl Session {
    /// # Errors
    ///
    /// When the terminal cannot be set up, the program cannot be started or
    /// a thread cannot be created. The message names the stage.
    pub fn spawn(command: &PtyCommand) -> io::Result<Self> {
        let (child, master) = spawn::spawn(command)?;
        let (sender, output) = mpsc::sync_channel(OUTPUT_QUEUE);
        let mut session = Self {
            child,
            master,
            output,
            writer: None,
            cancellation: Cancellation::default(),
            exit: None,
        };

        // On failure the session is dropped, which kills the program.
        session
            .master
            .try_clone()
            .and_then(|reading_end| reader::start(reading_end, sender))
            .map_err(|error| spawn::stage_error("read the terminal", error))?;

        session.writer = Some(
            session
                .master
                .try_clone()
                .and_then(Writer::start)
                .map_err(|error| spawn::stage_error("write to the terminal", error))?,
        );

        tracing::debug!(
            "started {} in a pseudo-terminal (pid {})",
            command.program().display(),
            session.id()
        );

        Ok(session)
    }

    /// The program's process id, which is also its process group id.
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    /// Ends with [`Chunk::Closed`].
    pub const fn output(&self) -> &Receiver<Chunk> {
        &self.output
    }

    /// Returns at once: the writer thread waits for a program that does not read,
    /// so the caller can still cancel it.
    ///
    /// # Errors
    ///
    /// `WouldBlock` when too much input already waits for the program,
    /// `BrokenPipe` after a cancel or once the terminal is closed.
    pub fn input(&self, data: &str) -> io::Result<()> {
        self.writer.as_ref().map_or_else(
            || Err(io::ErrorKind::BrokenPipe.into()),
            |writer| writer.send(data.as_bytes()),
        )
    }

    /// The kernel sends the program SIGWINCH.
    ///
    /// # Errors
    ///
    /// When the terminal is closed.
    pub fn resize(&self, size: TerminalSize) -> io::Result<()> {
        Ok(tcsetwinsize(&self.master, winsize(clamped(size)))?)
    }

    /// Keep calling [`Self::poll_cancel`] until the program exited, so harder
    /// signals follow when it does not stop.
    ///
    /// # Errors
    ///
    /// When the signal cannot be sent.
    pub fn cancel(&mut self, now: Instant) -> io::Result<()> {
        if let Some(writer) = &self.writer {
            writer.abandon();
        }

        let signal = self.cancellation.start(now);
        self.send(signal)
    }

    /// # Errors
    ///
    /// When the signal cannot be sent.
    pub fn poll_cancel(&mut self, now: Instant) -> io::Result<()> {
        let signal = self.cancellation.poll(now);
        self.send(signal)
    }

    pub const fn is_cancelled(&self) -> bool {
        self.cancellation.is_requested()
    }

    fn send(&self, signal: Option<Signal>) -> io::Result<()> {
        signal
            .filter(|_| self.exit.is_none())
            .map_or(Ok(()), |signal| self.signal_group(signal))
    }

    fn signal_group(&self, signal: Signal) -> io::Result<()> {
        tracing::debug!("sending {signal:?} to process group {}", self.id());
        match kill_process_group(Pid::from_child(&self.child), signal.to_rustix()) {
            // The group is gone already.
            Err(rustix::io::Errno::SRCH) => Ok(()),
            result => Ok(result?),
        }
    }

    /// Call it in a loop with [`Self::poll_cancel`], a blocking wait could not cancel.
    ///
    /// The leader is reaped only after the group got its kill. A reaped
    /// leader frees its group id, which another program may take, so a
    /// cancel kills the group while the leader still waits as a zombie.
    /// Children that outlive the program (a hook ignoring SIGINT and SIGHUP)
    /// are killed here.
    ///
    /// # Errors
    ///
    /// When waiting on the child fails.
    pub fn try_wait(&mut self) -> io::Result<Option<Exit>> {
        if self.exit.is_some() {
            return Ok(self.exit);
        }

        if !has_ended(&self.child)? {
            return Ok(None);
        }

        if self.is_cancelled()
            && let Err(error) = self.signal_group(Signal::Kill)
        {
            tracing::warn!("cannot kill what is left of {}: {error}", self.id());
        }

        let exit = Exit::from(self.child.wait()?);

        self.exit = Some(exit);
        Ok(Some(exit))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.exit.is_some() {
            return;
        }

        if let Err(error) = self.send(Some(Signal::Kill)) {
            tracing::warn!("cannot kill the abandoned program {}: {error}", self.id());
        }

        // Reaping keeps no zombie behind.
        self.child.wait().ok();
    }
}

/// Whether the program has ended, without reaping it. A zombie keeps its
/// process id, and with it the group id, until the caller reaps it.
///
/// # Errors
///
/// When the status cannot be read.
pub fn has_ended(child: &Child) -> io::Result<bool> {
    let options = WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG;

    Ok(waitid(WaitId::Pid(Pid::from_child(child)), options)?.is_some())
}
