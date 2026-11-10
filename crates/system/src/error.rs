//! How a probe fails.

use std::fmt::Display;
use std::io;
use std::sync::Arc;

#[derive(Clone, Debug, thiserror::Error)]
pub enum ProbeError {
    /// The program or file does not exist on this system, for example
    /// pacman on a distribution without it. The app shows the part as not
    /// available instead of as an error.
    #[error("not available on this system")]
    Unavailable,
    /// The source exists but gave no usable answer.
    #[error("probe failed: {0}")]
    Failed(String),
    /// Reading `what` failed. Shared so the error stays `Clone`.
    #[error("probe failed: {what}: {error}")]
    Io {
        what: String,
        #[source]
        error: Arc<io::Error>,
    },
}

impl ProbeError {
    /// What went wrong in words, `None` when the source is not there.
    pub fn reason(&self) -> Option<String> {
        match self {
            Self::Unavailable => None,
            Self::Failed(reason) => Some(reason.clone()),
            Self::Io { what, error } => Some(format!("{what}: {error}")),
        }
    }

    pub const fn is_failure(&self) -> bool {
        !matches!(self, Self::Unavailable)
    }
}

/// An I/O error has no equality of its own, so two count as equal when
/// they read the same.
impl PartialEq for ProbeError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Io { what, error },
                Self::Io {
                    what: other_what,
                    error: other_error,
                },
            ) => {
                what == other_what
                    && error.kind() == other_error.kind()
                    && error.to_string() == other_error.to_string()
            }
            (Self::Unavailable, Self::Unavailable) => true,
            (Self::Failed(reason), Self::Failed(other_reason)) => reason == other_reason,
            (
                Self::Unavailable | Self::Failed(_) | Self::Io { what: _, error: _ },
                Self::Unavailable | Self::Failed(_) | Self::Io { what: _, error: _ },
            ) => false,
        }
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
