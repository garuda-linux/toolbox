//! `rate-mirrors` runs as the user into a temporary file, and the result
//! is validated as a [`Mirrorlist`] before the helper may install it as
//! root. Blocks, so call it off the GUI thread.

#[cfg(test)]
mod tests;

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use toolbox_exec::{CaptureError, Environment, Policy, run_captured};
use toolbox_plan::Mirrorlist;

/// Where Garuda installs `rate-mirrors`.
const PROGRAM: &str = "/usr/bin/rate-mirrors";

/// Ranking tests every mirror. It may take a while, never forever.
const RANKING_DEADLINE: Duration = Duration::from_mins(5);

/// What `rate-mirrors` prints while it measures. Only the saved file
/// counts, so a little of its output is plenty.
const KEPT_OUTPUT_BYTES: usize = 64 << 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RankProblem {
    TimedOut,
    NotStarted(String),
    Failed(String),
    Unreadable(String),
    Invalid(String),
}

impl fmt::Display for RankProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TimedOut => f.write_str("rate-mirrors took too long"),
            Self::NotStarted(reason) => write!(f, "cannot start rate-mirrors: {reason}"),
            Self::Failed(status) => write!(f, "rate-mirrors ended with {status}"),
            Self::Unreadable(reason) => write!(f, "cannot read the ranked mirrors: {reason}"),
            Self::Invalid(reason) => write!(f, "the ranked mirrors are unusable: {reason}"),
        }
    }
}

impl RankProblem {
    /// What the system said, empty for a ranking that took too long.
    pub fn reason(&self) -> &str {
        match self {
            Self::TimedOut => "",
            Self::NotStarted(reason)
            | Self::Failed(reason)
            | Self::Unreadable(reason)
            | Self::Invalid(reason) => reason,
        }
    }
}

pub trait MirrorRanking: Send + Sync {
    /// Blocks for minutes, so call it off the GUI thread.
    ///
    /// # Errors
    fn rank(&self) -> Result<Mirrorlist, RankProblem>;
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
    fn rank(&self) -> Result<Mirrorlist, RankProblem> {
        rank_within(&self.program, RANKING_DEADLINE)
    }
}

/// `deadline` is a parameter so tests need not wait minutes.
fn rank_within(program: &Path, deadline: Duration) -> Result<Mirrorlist, RankProblem> {
    let folder = tempfile::tempdir().map_err(|error| RankProblem::NotStarted(error.to_string()))?;
    let file = folder.path().join("chaotic-mirrorlist");
    let mut command = Command::new(program);

    command.arg("--save").arg(&file).arg("chaotic-aur");

    let exited = run_captured(
        &mut command,
        Environment::Inherited,
        &[],
        &Policy::tail(KEPT_OUTPUT_BYTES).within(deadline),
        &Arc::default(),
    )
    .map_err(|error| match error {
        CaptureError::TimedOut => RankProblem::TimedOut,
        CaptureError::Missing | CaptureError::Failed(_) => {
            RankProblem::NotStarted(error.to_string())
        }
        CaptureError::Cancelled | CaptureError::TooLarge => RankProblem::Failed(error.to_string()),
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
