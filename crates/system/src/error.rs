//! How a probe fails.

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::fmt::{self, Display};
use std::io;
use std::mem::discriminant;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::sync::Arc;
use std::time::Duration;

use toolbox_plan::InvalidName;
use toolbox_state::bootrepair::InvalidListing;

/// Every failure but [`ProbeError::Unavailable`] reads `probe failed: `
/// and its [`ProbeError::reason`].
#[derive(Clone, Debug)]
pub enum ProbeError {
    /// The program or file does not exist on this system, for example
    /// pacman on a distribution without it. The app shows the part as not
    /// available instead of as an error.
    Unavailable,
    /// The program ended with an error.
    Command {
        command_line: String,
        status: ExitStatus,
        /// Its first line, without escape sequences.
        stderr: String,
    },
    /// The program gave no answer within `after` and was stopped.
    TimedOut {
        command_line: String,
        after: Duration,
    },
    Stopped {
        command_line: String,
        why: Stop,
    },
    /// Shared so the error stays `Clone`.
    Json {
        what: String,
        error: Arc<serde_json::Error>,
    },
    Invalid {
        what: String,
        error: InvalidValue,
    },
    Alpm {
        what: String,
        error: alpm::Error,
    },
    Syscall {
        what: String,
        error: rustix::io::Errno,
    },
    NotUtf8 {
        what: String,
        raw: OsString,
    },
    /// A source that exists names nothing where it should.
    Missing(MissingFact),
    NotAFile {
        path: PathBuf,
    },
    /// libalpm takes C strings, which end at a NUL.
    NulInPath {
        path: PathBuf,
    },
    /// The thread or task that read the part panicked.
    Panicked,
    /// Reading `what` failed. Shared so the error stays `Clone`.
    Io {
        what: String,
        error: Arc<io::Error>,
    },
}

/// Why a program was stopped before its end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Cancelled,
    TooLarge,
}

/// What a source that exists lacks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MissingFact {
    /// The effective user id is not in the process status file.
    UserId { path: PathBuf },
    /// The mount table lists no root filesystem.
    Root { path: PathBuf },
    /// The kernel statistics have no boot time.
    BootTime,
    /// The OS release file names no kernel release.
    KernelRelease,
}

impl MissingFact {
    /// The reason in words, a whole sentence.
    fn reason(&self) -> String {
        match self {
            Self::UserId { path } => format!("{} names no user id", path.display()),
            Self::Root { path } => format!("{} lists no root", path.display()),
            Self::BootTime => "no boot time in /proc/stat".to_owned(),
            Self::KernelRelease => "the kernel names no release".to_owned(),
        }
    }
}

/// A value the source printed that the workspace's parsers refuse.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidValue {
    #[error(transparent)]
    Name(#[from] InvalidName),
    #[error(transparent)]
    Listing(#[from] InvalidListing),
}

impl ProbeError {
    /// What went wrong in words, `None` when the source is not there.
    pub fn reason(&self) -> Option<String> {
        let reason = match self {
            Self::Unavailable => return None,
            Self::Command {
                command_line,
                status,
                stderr,
            } => format!("{command_line} {status}: {stderr}"),
            Self::TimedOut {
                command_line,
                after,
            } => format!(
                "{command_line}: no answer within {} s, stopped",
                after.as_secs_f32()
            ),
            Self::Stopped { command_line, why } => format!("{command_line}: {why}"),
            Self::Json { what, error } => format!("{what}: {error}"),
            Self::Invalid { what, error } => format!("{what}: {error}"),
            Self::Alpm { what, error } => format!("{what}: {error}"),
            Self::Syscall { what, error } => format!("{what}: {error}"),
            Self::NotUtf8 { what, raw } => format!("{what} is not UTF-8: {}", raw.display()),
            Self::Missing(fact) => fact.reason(),
            Self::NotAFile { path } => format!("{} is no file", path.display()),
            Self::NulInPath { path } => format!("{}: NUL in the path", path.display()),
            Self::Panicked => "the probe panicked".to_owned(),
            Self::Io { what, error } => format!("{what}: {error}"),
        };

        Some(reason)
    }

    pub const fn is_failure(&self) -> bool {
        !matches!(self, Self::Unavailable)
    }

    /// A failed read of `what`, for fakes that stand in for a source.
    #[cfg(any(test, feature = "testing"))]
    pub fn fake(what: impl Into<String>) -> Self {
        Self::Io {
            what: what.into(),
            error: Arc::new(io::Error::other("fake source")),
        }
    }
}

impl Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason() {
            None => f.write_str("not available on this system"),
            Some(reason) => write!(f, "probe failed: {reason}"),
        }
    }
}

impl std::error::Error for ProbeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json { what: _, error } => Some(&**error),
            Self::Invalid { what: _, error } => Some(error),
            Self::Alpm { what: _, error } => Some(error),
            Self::Syscall { what: _, error } => Some(error),
            Self::Io { what: _, error } => Some(&**error),
            Self::Unavailable
            | Self::Command {
                command_line: _,
                status: _,
                stderr: _,
            }
            | Self::TimedOut {
                command_line: _,
                after: _,
            }
            | Self::Stopped {
                command_line: _,
                why: _,
            }
            | Self::NotUtf8 { what: _, raw: _ }
            | Self::Missing(_)
            | Self::NotAFile { path: _ }
            | Self::NulInPath { path: _ }
            | Self::Panicked => None,
        }
    }
}

impl Display for Stop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "cancelled",
            Self::TooLarge => "the program printed too much",
        })
    }
}

/// An I/O or JSON error has no equality of its own, so two failures count
/// as equal when they are of one kind and read the same.
impl PartialEq for ProbeError {
    fn eq(&self, other: &Self) -> bool {
        let kinds = match (self, other) {
            (
                Self::Io { what: _, error },
                Self::Io {
                    what: _,
                    error: theirs,
                },
            ) => error.kind() == theirs.kind(),
            (ours, theirs) => discriminant(ours) == discriminant(theirs),
        };

        kinds && self.reason() == other.reason()
    }
}

impl Eq for ProbeError {}

/// Sorts an I/O error into the two kinds a caller tells apart.
pub fn from_io<W>(what: W, error: io::Error) -> ProbeError
where
    W: Display,
{
    if error.kind() == io::ErrorKind::NotFound {
        ProbeError::Unavailable
    } else {
        ProbeError::Io {
            what: what.to_string(),
            error: Arc::new(error),
        }
    }
}
