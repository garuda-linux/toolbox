//! Configuration files waiting for a merge: `.pacnew`, `.pacsave` and
//! `.pacorig` files, as `pacdiff -o` lists them.

#[cfg(any(test, feature = "selftest"))]
mod found;
#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::path::PathBuf;

use toolbox_exec::{Environment, locate_program};
use toolbox_plan::MergeCopy;
use toolbox_plan::procedure::Program;
use toolbox_state::parts::MergeCopies;

#[cfg(any(test, feature = "selftest"))]
pub use self::found::FoundCopies;
use crate::command;
use crate::error::ProbeError;

/// Answers which configuration files wait for a merge.
pub trait MergeSource: Send + Sync {
    /// The waiting files (the `.pacnew` and similar, not the originals).
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without the program,
    /// A failed [`ProbeError`] when it fails.
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
    /// `pacdiff` at its fixed path.
    #[must_use = "the adapter does nothing until asked"]
    pub fn new() -> Self {
        Self::with_program(locate_program(Program::Pacdiff, Environment::Inherited))
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

/// The copies the toolbox may open. Others, such as `/etc/shadow.pacnew`,
/// stay for a merge in a terminal.
///
/// # Errors
///
/// The [`ProbeError`] of `source`.
pub fn merge_copies(source: &dyn MergeSource) -> Result<MergeCopies, ProbeError> {
    source.waiting().map(|paths| {
        MergeCopies(
            paths
                .iter()
                .filter_map(|path| path.to_str())
                .filter_map(|path| MergeCopy::new(path).ok())
                .collect(),
        )
    })
}
