//! Configuration files waiting for a merge: `.pacnew`, `.pacsave` and
//! `.pacorig` files, as `pacdiff -o` lists them.

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::path::PathBuf;

use crate::command;
use crate::error::ProbeError;

/// Answers which configuration files wait for a merge.
pub trait MergeSource: Send + Sync {
    /// The waiting files (the `.pacnew` and similar, not the originals).
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without the program,
    /// [`ProbeError::Failed`] when it fails.
    fn waiting(&self) -> Result<Vec<PathBuf>, ProbeError>;
}

/// [`MergeSource`] through `pacdiff` from pacman-contrib. With `-o` it only
/// prints and needs no root: it reads the backup lists in pacman's local
/// database, which every user may read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pacdiff {
    program: OsString,
}

impl Pacdiff {
    /// `pacdiff` from `PATH`.
    #[must_use = "the adapter does nothing until asked"]
    pub fn new() -> Self {
        Self::with_program("pacdiff")
    }

    /// Another program with pacdiff's interface, looked up like a command.
    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn with_program(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl Default for Pacdiff {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeSource for Pacdiff {
    /// pacdiff 1.13.1 exits with 0 after listing, 1 on bad options.
    fn waiting(&self) -> Result<Vec<PathBuf>, ProbeError> {
        command::output(&self.program, &["-o"]).map(|output| parse_files(&output))
    }
}

/// One absolute path per line. Paths keep their spaces, anything else is
/// a message and left out.
fn parse_files(output: &str) -> Vec<PathBuf> {
    output
        .lines()
        .filter(|line| line.starts_with('/'))
        .map(PathBuf::from)
        .collect()
}
