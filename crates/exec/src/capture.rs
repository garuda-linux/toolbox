//! Running a program without a terminal to its end and keeping what it
//! printed: the one runner for every such program of the helper and the
//! app. Plain pipes, so a program starts no pager and prints colours only
//! when asked to. A [`Policy`] says how long it may take, how it is
//! stopped, how much of its output is kept and what it reads.
//! A program the caller may not kill (one started through pkexec) ends on
//! its own once its pipes are closed.

mod process;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt as _;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};
use std::{fmt, io};

use crate::pty_runner::Environment;

/// A program's complaints are short. A flood of them is cut like the
/// output.
pub const MAX_ERROR_BYTES: usize = 64 << 10;

/// `PIPE_BUF`, what every pipe holds even when the user's pipes were
/// shrunk. Input is written before the wait, so it must never block.
pub const MAX_INPUT_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy<'input> {
    deadline: Option<Duration>,
    term_grace: Option<Duration>,
    limit: usize,
    error_limit: usize,
    cap_is_error: bool,
    stdin: Option<&'input [u8]>,
    identified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Captured {
    /// Lossy UTF-8, cut to the policy's limit.
    pub output: String,
    pub errors: String,
    /// `None` when a signal ended the program.
    pub code: Option<i32>,
    /// The start of the output was left out to stay below the limit.
    pub truncated: bool,
}

#[derive(Debug)]
pub struct Exited {
    pub status: ExitStatus,
    pub output: Vec<u8>,
    pub errors: Vec<u8>,
    pub truncated: bool,
}

#[derive(Debug)]
pub enum CaptureError {
    Missing,
    Failed(io::Error),
    Cancelled,
    TimedOut,
    /// It printed more than the limit of a policy that counts that as an
    /// error.
    TooLarge,
}

struct Tail {
    limit: usize,
    bytes: VecDeque<u8>,
    truncated: bool,
    /// The pipe broke before its end, so the bytes are no whole output.
    failed: Option<io::Error>,
}

impl<'input> Policy<'input> {
    #[must_use = "a policy is only described, nothing runs"]
    pub const fn tail(limit: usize) -> Self {
        Self {
            deadline: None,
            term_grace: None,
            limit,
            error_limit: MAX_ERROR_BYTES,
            cap_is_error: false,
            stdin: None,
            identified: false,
        }
    }

    /// All of both streams. For programs whose output is known to be
    /// small or that run with a deadline.
    #[must_use = "a policy is only described, nothing runs"]
    pub const fn whole() -> Self {
        Self {
            limit: usize::MAX,
            error_limit: usize::MAX,
            ..Self::tail(0)
        }
    }

    #[must_use = "a policy is only described, nothing runs"]
    pub const fn within(mut self, deadline: Duration) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// A stop sends SIGTERM first and SIGKILL after `grace`. pacman
    /// removes its database lock on SIGTERM, a SIGKILL leaves it behind.
    #[must_use = "a policy is only described, nothing runs"]
    pub const fn term_first(mut self, grace: Duration) -> Self {
        self.term_grace = Some(grace);
        self
    }

    /// Output past the limit stops the program and is an error, for an
    /// answer that is parsed: a cut one would read as a short one.
    #[must_use = "a policy is only described, nothing runs"]
    pub const fn capped(mut self) -> Self {
        self.cap_is_error = true;
        self
    }

    #[must_use = "a policy is only described, nothing runs"]
    pub const fn fed(mut self, input: &'input [u8]) -> Self {
        self.stdin = Some(input);
        self
    }

    /// Stdin is one end of a socket pair, so the program learns this
    /// process's uid from the kernel, which no caller can fake. Nothing is
    /// sent on it. Only without input.
    #[must_use = "a policy is only described, nothing runs"]
    pub const fn identified(mut self) -> Self {
        self.identified = true;
        self
    }

    /// The deadline as a point in time. One too far away never passes.
    fn end(&self, started: Instant) -> Option<Instant> {
        started.checked_add(self.deadline?)
    }
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("the program is not installed"),
            Self::Failed(error) => write!(f, "{error}"),
            Self::Cancelled => f.write_str("cancelled"),
            Self::TimedOut => f.write_str("the program took too long"),
            Self::TooLarge => f.write_str("the program printed too much"),
        }
    }
}

impl std::error::Error for CaptureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Failed(error) => Some(error),
            Self::Missing | Self::Cancelled | Self::TimedOut | Self::TooLarge => None,
        }
    }
}

impl From<CaptureError> for io::Error {
    fn from(error: CaptureError) -> Self {
        let kind = match error {
            CaptureError::Failed(error) => return error,
            CaptureError::Missing => io::ErrorKind::NotFound,
            CaptureError::TimedOut => io::ErrorKind::TimedOut,
            CaptureError::Cancelled | CaptureError::TooLarge => io::ErrorKind::Other,
        };

        Self::new(kind, error.to_string())
    }
}

impl Tail {
    const fn new(limit: usize) -> Self {
        Self {
            limit,
            bytes: VecDeque::new(),
            truncated: false,
            failed: None,
        }
    }

    fn push(&mut self, chunk: &[u8]) {
        self.bytes.extend(chunk);

        let excess = self.bytes.len().saturating_sub(self.limit);

        if excess > 0 {
            self.bytes.drain(..excess);
            self.truncated = true;
        }
    }

    /// The kept bytes. A cut output starts at its first whole line, so no
    /// half line and no half character leads the report.
    fn into_bytes(self) -> (Vec<u8>, bool) {
        let mut bytes: Vec<u8> = self.bytes.into();
        let first_line = bytes
            .iter()
            .position(|byte| *byte == b'\n')
            .filter(|_| self.truncated);

        if let Some(end) = first_line {
            bytes.drain(..=end);
        }

        (bytes, self.truncated)
    }
}

impl From<Exited> for Captured {
    fn from(exited: Exited) -> Self {
        Self {
            output: String::from_utf8_lossy(&exited.output).into_owned(),
            errors: String::from_utf8_lossy(&exited.errors).into_owned(),
            code: exited.status.code(),
            truncated: exited.truncated,
        }
    }
}

/// # Errors
///
/// As [`run_captured`].
pub fn capture(
    command: &mut Command,
    environment: Environment,
    variables: &[(&str, &str)],
    policy: &Policy<'_>,
    cancel: &Arc<AtomicBool>,
) -> Result<Captured, CaptureError> {
    run_captured(command, environment, variables, policy, cancel).map(Captured::from)
}

/// `cancel` stops the wait and kills the process group. `variables` join
/// the environment, a clean one too: they are the caller's fixed choice.
///
/// # Errors
///
/// [`CaptureError::Missing`] for a program that does not exist,
/// [`CaptureError::Cancelled`] after a cancel, [`CaptureError::TimedOut`]
/// past the deadline, [`CaptureError::TooLarge`] past the limit of a
/// capped policy, [`CaptureError::Failed`] when it cannot be started, fed
/// or waited for.
pub fn run_captured(
    command: &mut Command,
    environment: Environment,
    variables: &[(&str, &str)],
    policy: &Policy<'_>,
    cancel: &Arc<AtomicBool>,
) -> Result<Exited, CaptureError> {
    if policy
        .stdin
        .is_some_and(|input| input.len() > MAX_INPUT_BYTES)
    {
        return Err(CaptureError::Failed(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the input is longer than a pipe surely holds",
        )));
    }

    // Held until the program ends, so its stdin stays a connected socket.
    let mut ours = None;
    let stdin = match (policy.stdin, policy.identified) {
        (Some(_), _) => Stdio::piped(),
        (None, true) => {
            let (kept, theirs) = UnixStream::pair().map_err(CaptureError::Failed)?;

            ours = Some(kept);
            Stdio::from(OwnedFd::from(theirs))
        }
        (None, false) => Stdio::null(),
    };

    command
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Its own group, so a stop also reaches what the program started.
        .process_group(0);

    if environment == Environment::Clean {
        command.env_clear();
    }

    command.envs(environment.variables().iter().chain(variables).copied());

    let started = Instant::now();
    let child = command.spawn().map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            CaptureError::Missing
        } else {
            CaptureError::Failed(error)
        }
    })?;

    let exited = process::finish(child, policy, started, cancel);

    drop(ours);
    exited
}
