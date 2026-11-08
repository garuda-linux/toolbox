//! How a probe fails.

use std::fmt::Display;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ProbeError {
    /// The program or file does not exist on this system, for example
    /// pacman on a distribution without it. The app shows the part as not
    /// available instead of as an error.
    #[error("not available on this system")]
    Unavailable,
    /// The source exists but gave no usable answer.
    #[error("probe failed: {0}")]
    Failed(String),
}

/// Sorts an I/O error into the two kinds a caller tells apart.
pub fn from_io<W>(what: W, error: &io::Error) -> ProbeError
where
    W: Display,
{
    if error.kind() == io::ErrorKind::NotFound {
        ProbeError::Unavailable
    } else {
        ProbeError::Failed(format!("{what}: {error}"))
    }
}
