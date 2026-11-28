//! `rate-mirrors` runs as the user into a temporary file, and the result
//! is validated as a [`Mirrorlist`] before the helper may install it as
//! root. Blocks, so call it off the GUI thread.

#[cfg(test)]
mod tests;

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use toolbox_exec::{CaptureError, Environment, Policy, run_captured};
use toolbox_plan::Mirrorlist;

const PROGRAM: &str = "/usr/bin/rate-mirrors";

/// Ranking tests every mirror. It may take a while, never forever.
const RANKING_DEADLINE: Duration = Duration::from_mins(5);

/// What `rate-mirrors` prints while it measures. Only the saved file
/// counts, so a little of its output is plenty.
const KEPT_OUTPUT_BYTES: usize = 64 << 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RankProblem {
    TimedOut,
    /// The user stopped the ranking, so nothing is wrong.
    Cancelled,
    NotStarted(StartProblem),
    Failed(String),
    Unreadable(String),
    Invalid(String),
}

/// Why rate-mirrors did not start. The window words each case, and only
/// [`StartProblem::Failed`] carries the system's own words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartProblem {
    /// The program is not installed.
    Missing,
    /// The system's words, for the program or the folder that did not start.
    Failed(String),
    /// The task that waited for the ranking panicked.
    Panicked,
}

impl fmt::Display for StartProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("the program is not installed"),
            Self::Failed(words) => f.write_str(words),
            Self::Panicked => f.write_str("the ranking task panicked"),
        }
    }
}

impl fmt::Display for RankProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimedOut => f.write_str("rate-mirrors took too long"),
            Self::Cancelled => f.write_str("rate-mirrors was stopped"),
            Self::NotStarted(start) => write!(f, "cannot start rate-mirrors: {start}"),
            Self::Failed(status) => write!(f, "rate-mirrors ended with {status}"),
            Self::Unreadable(reason) => write!(f, "cannot read the ranked mirrors: {reason}"),
            Self::Invalid(reason) => write!(f, "the ranked mirrors are unusable: {reason}"),
        }
    }
}

pub trait MirrorRanking: Send + Sync {
    /// Blocks for minutes, so call it off the GUI thread. Setting `cancel`
    /// kills rate-mirrors and returns [`RankProblem::Cancelled`].
    ///
    /// # Errors
    fn rank(&self, cancel: &Arc<AtomicBool>) -> Result<Mirrorlist, RankProblem>;
}

pub struct RateMirrors {
    program: PathBuf,
}

impl RateMirrors {
    #[must_use = "the ranking does nothing until it is asked"]
    pub fn system() -> Self {
        Self::new(PathBuf::from(PROGRAM))
    }

    /// `program` is a stand-in in tests.
    #[must_use = "the ranking does nothing until it is asked"]
    pub(crate) const fn new(program: PathBuf) -> Self {
        Self { program }
    }
}

impl MirrorRanking for RateMirrors {
    fn rank(&self, cancel: &Arc<AtomicBool>) -> Result<Mirrorlist, RankProblem> {
        rank_within(&self.program, RANKING_DEADLINE, cancel)
    }
}

/// `deadline` is a parameter so tests need not wait minutes.
fn rank_within(
    program: &Path,
    deadline: Duration,
    cancel: &Arc<AtomicBool>,
) -> Result<Mirrorlist, RankProblem> {
    let folder = tempfile::tempdir()
        .map_err(|error| RankProblem::NotStarted(StartProblem::Failed(error.to_string())))?;

    let file = folder.path().join("chaotic-mirrorlist");

    let mut command = Command::new(program);

    command.arg("--save").arg(&file).arg("chaotic-aur");

    let exited = run_captured(
        &mut command,
        Environment::Inherited,
        &[],
        &Policy::tail(KEPT_OUTPUT_BYTES).within(deadline),
        cancel,
    )
    .map_err(|error| match error {
        CaptureError::TimedOut => RankProblem::TimedOut,
        CaptureError::Cancelled => RankProblem::Cancelled,
        CaptureError::Missing => RankProblem::NotStarted(StartProblem::Missing),
        CaptureError::Failed(start) => {
            RankProblem::NotStarted(StartProblem::Failed(start.to_string()))
        }
        CaptureError::TooLarge => RankProblem::Failed(CaptureError::TooLarge.to_string()),
    })?;

    if !exited.status.success() {
        return Err(RankProblem::Failed(format!(
            "{}: {}",
            exited.status,
            stderr_tail(&exited.errors)
        )));
    }

    let text = std::fs::read_to_string(&file)
        .map_err(|error| RankProblem::Unreadable(error.to_string()))?;

    Mirrorlist::new(text).map_err(|error| RankProblem::Invalid(error.to_string()))
}

/// The end of stderr, where the reason for a failure usually is. The
/// whole stream may hold a mirror per line, so only the last part is
/// kept for the page.
fn stderr_tail(stderr: &[u8]) -> String {
    const KEEP: usize = 2048;

    let start = stderr.len().saturating_sub(KEEP);
    let kept = stderr.get(start..).unwrap_or(stderr);

    String::from_utf8_lossy(kept).into_owned()
}
