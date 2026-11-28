//! A query of a step: a program without a terminal whose output lines
//! become the targets of the next command (`pacman -Qtdq` lists orphans).
//! It runs as root in the helper, so its output is capped and a stop
//! ends it.

#[cfg(test)]
mod tests;

use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use toolbox_plan::procedure::Command;

use super::{Environment, LOCALE};
use crate::capture::{CaptureError, Policy, capture};

/// Package names, one per line. Far more than every package of a
/// repository, far less than a root process should hold for nothing.
const MAX_QUERY_BYTES: usize = 8 << 20;

/// A query lists what the database holds, which takes seconds. One that
/// waits for a lock or a device would hold the helper without an end.
const QUERY_DEADLINE: Duration = Duration::from_mins(2);

/// Which ends of a query are an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryKind {
    Listing,
    /// `pacman -T`: exit code 0 when every name is met, 127 with the
    /// unmet names.
    Deptest,
    /// `pacman -Qdtq -- <names>`: the names that are orphans. Exit code 1
    /// also says that a name is not installed, on stderr.
    Orphans,
}

impl QueryKind {
    fn answered(self, code: Option<i32>, lines: &[String], errors: &str, asked: &[String]) -> bool {
        let quiet = lines.is_empty() && errors.is_empty();
        match self {
            Self::Listing => code == Some(0_i32) || empty_miss(code, quiet),
            Self::Deptest => code == Some(0_i32) || (code == Some(127_i32) && !lines.is_empty()),
            Self::Orphans => orphans_answered(code, lines, errors, asked),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum QueryError {
    #[error("{command} printed more than {limit} bytes")]
    TooLarge { command: Command, limit: usize },
    #[error("{command} did not end within {seconds} seconds")]
    TimedOut { command: Command, seconds: u64 },
    #[error("{command} ended with {ended}: {errors}")]
    Ended {
        command: Command,
        ended: String,
        errors: String,
    },
}

/// Exit 0 with the orphans, or exit 1 when none is one, where every error
/// line says a name was not found and that name was asked. Exit 1 with
/// orphans and no such error line is no answer. At exit 0 pacman may still
/// warn, such as about its own config, which does not change the answer.
fn orphans_answered(code: Option<i32>, lines: &[String], errors: &str, asked: &[String]) -> bool {
    let not_found_of_asked = errors.lines().all(|line| {
        not_found(line).is_some_and(|name| asked.iter().any(|candidate| candidate == name))
    });

    match code {
        Some(0_i32) => errors.lines().all(|line| line.starts_with("warning: ")),
        Some(1_i32) => not_found_of_asked && (lines.is_empty() || !errors.is_empty()),
        _ => false,
    }
}

fn not_found(line: &str) -> Option<&str> {
    line.strip_prefix("error: package '")?
        .strip_suffix("' was not found")
}

/// Runs `command` with `program` in place of its own (a parameter so
/// tests can run a stand-in) and returns its output lines. A failure
/// counts as an answer only where `kind` says so, otherwise it keeps the
/// program's own words.
///
/// # Errors
///
/// When the program cannot start, is cancelled, prints more than
/// [`MAX_QUERY_BYTES`] or fails.
pub fn run(
    program: impl AsRef<std::ffi::OsStr>,
    command: &Command,
    kind: QueryKind,
    environment: Environment,
    cancel: &Arc<AtomicBool>,
) -> io::Result<Vec<String>> {
    let mut query = std::process::Command::new(program);

    query.args(command.arguments());

    // English output on both sides: the lines are parsed, not shown. The
    // command's own variables come after, so they win.
    let variables: Vec<(&str, &str)> = std::iter::once(("LANG", LOCALE))
        .chain(command.variables().iter().copied())
        .collect();

    let policy = Policy::tail(MAX_QUERY_BYTES)
        .capped()
        .within(QUERY_DEADLINE);

    let captured = capture(&mut query, environment, &variables, &policy, cancel).map_err(
        |error| match error {
            CaptureError::TooLarge => io::Error::other(QueryError::TooLarge {
                command: command.clone(),
                limit: MAX_QUERY_BYTES,
            }),
            CaptureError::TimedOut => io::Error::new(
                io::ErrorKind::TimedOut,
                QueryError::TimedOut {
                    command: command.clone(),
                    seconds: QUERY_DEADLINE.as_secs(),
                },
            ),
            other @ (CaptureError::Failed(_) | CaptureError::Missing | CaptureError::Cancelled) => {
                other.into()
            }
        },
    )?;

    let lines: Vec<String> = captured
        .output
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();

    if kind.answered(
        captured.code,
        &lines,
        captured.errors.trim(),
        command.arguments(),
    ) {
        return Ok(lines);
    }

    let ended = captured
        .code
        .map_or_else(|| "a signal".to_owned(), |code| format!("exit code {code}"));

    Err(io::Error::other(QueryError::Ended {
        command: command.clone(),
        ended,
        errors: captured.errors.trim().to_owned(),
    }))
}

/// Only exit code 1 with no output and no error text is a miss: pacman
/// `-Qtdq` exits with 1 when there are no orphans. Every other failure
/// keeps its error, so a real problem never reads as an empty answer.
fn empty_miss(code: Option<i32>, quiet: bool) -> bool {
    code == Some(1_i32) && quiet
}
