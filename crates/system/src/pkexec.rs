//! What it means when `pkexec` did not run the root helper. Every caller
//! of the helper asks here, so a closed dialog reads the same everywhere.

#[cfg(test)]
mod tests;

use std::fmt;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_exec::{CaptureError, Captured, Environment, Policy, capture};

const DISMISSED: i32 = 126;
/// Every other failure of pkexec, told apart by its words on stderr.
const FAILED: i32 = 127;

/// The textual fallback agent never runs without a terminal, and without
/// a session agent no dialog appears at all.
const NO_AGENT_MARKS: [&str; 6] = [
    "no authentication agent",
    "textual authentication agent",
    "local authentication agent",
    "error getting authority",
    "error checking for authorization",
    "no session",
];

const HELPER_MISSING_MARK: &str = "cannot run program";
/// A wrong password or a user who is no administrator.
const DENIED_MARK: &str = "not authorized";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Dismissed,
    /// polkit said no: wrong password, or no administrator.
    Denied,
    NoAgent,
    HelperMissing,
    /// pkexec itself cannot work here, for example a copy that is not
    /// setuid root. Its own words say why, see [`reason`].
    Broken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelperRun {
    /// Its answer line, as lossy UTF-8.
    Answered(String),
    Refused(Refusal),
    /// It could not start, failed or printed more than the limit. The
    /// text names why, with the helper's own words where it has some.
    Failed(String),
    Cancelled,
}

impl Refusal {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Dismissed => "dismissed",
            Self::Denied => "denied",
            Self::NoAgent => "no-agent",
            Self::HelperMissing => "helper-missing",
            Self::Broken => "broken",
        }
    }

    pub const fn not_authorized(self) -> bool {
        matches!(self, Self::Dismissed | Self::Denied)
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Dismissed => "the password dialog was dismissed",
            Self::Denied => "the authorization was denied",
            Self::NoAgent => "no password dialog could appear",
            Self::HelperMissing => "the helper of Garuda Toolbox is not installed",
            Self::Broken => "pkexec cannot run on this system",
        })
    }
}

/// At most `limit` bytes of its answer are kept, a longer one fails. No
/// deadline: an unanswered password dialog waits for its user, and only
/// `cancel` ends that wait.
pub fn ask(command: &mut Command, limit: usize, cancel: &Arc<AtomicBool>) -> HelperRun {
    match capture(
        command,
        Environment::Inherited,
        &[],
        &Policy::tail(limit),
        cancel,
    ) {
        Ok(captured) => sorted(captured),
        Err(CaptureError::Missing) => {
            HelperRun::Failed("the helper of Garuda Toolbox or pkexec is not installed".to_owned())
        }
        Err(CaptureError::Failed(error)) => {
            HelperRun::Failed(format!("cannot start the helper: {error}"))
        }
        Err(CaptureError::TimedOut) => HelperRun::Failed("the helper took too long".to_owned()),
        Err(CaptureError::TooLarge) => {
            HelperRun::Failed("the helper's answer is too long".to_owned())
        }
        Err(CaptureError::Cancelled) => HelperRun::Cancelled,
    }
}

fn sorted(captured: Captured) -> HelperRun {
    match refusal(captured.code, &captured.errors) {
        Some(Refusal::Broken) => return HelperRun::Failed(broken(&captured.errors)),
        Some(refused) => return HelperRun::Refused(refused),
        None => {}
    }

    match captured.code {
        Some(0) if captured.truncated => {
            HelperRun::Failed("the helper's answer is too long".to_owned())
        }
        Some(0) => HelperRun::Answered(captured.output),
        _ => HelperRun::Failed(ended(captured.code, &captured.errors)),
    }
}

pub(crate) fn broken(stderr: &str) -> String {
    format!("pkexec cannot run: {}", reason(stderr))
}

pub(crate) fn ended(code: Option<i32>, stderr: &str) -> String {
    let ending = code.map_or_else(|| "a signal".to_owned(), |code| format!("exit code {code}"));

    format!("the helper ended with {ending}: {}", stderr.trim())
}

/// `None` when the helper ran. Silence with 127 is a denial, which polkit
/// keeps quiet about. Unknown words mean pkexec itself failed.
pub fn refusal(code: Option<i32>, stderr: &str) -> Option<Refusal> {
    match code? {
        DISMISSED => Some(Refusal::Dismissed),
        FAILED => {
            let words = stderr.to_lowercase();

            Some(if NO_AGENT_MARKS.iter().any(|mark| words.contains(mark)) {
                Refusal::NoAgent
            } else if words.contains(HELPER_MISSING_MARK) {
                Refusal::HelperMissing
            } else if words.trim().is_empty() || words.contains(DENIED_MARK) {
                Refusal::Denied
            } else {
                Refusal::Broken
            })
        }
        _ => None,
    }
}

pub fn reason(stderr: &str) -> &str {
    stderr
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
}
