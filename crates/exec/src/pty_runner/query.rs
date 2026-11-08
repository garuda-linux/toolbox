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
pub const MAX_QUERY_BYTES: usize = 8 << 20;

/// A query lists what the database holds, which takes seconds. One that
/// waits for a lock or a device would hold the helper without an end.
pub const QUERY_DEADLINE: Duration = Duration::from_mins(2);

/// Runs `command` with `program` in place of its own (a parameter so
/// tests can run a stand-in) and returns its output lines. A failure
/// counts as nothing found only where [`empty_miss`] says so, otherwise
/// it keeps the program's own words.
///
/// # Errors
///
/// When the program cannot start, is cancelled, prints more than
/// [`MAX_QUERY_BYTES`] or fails.
pub fn run(
    program: impl AsRef<std::ffi::OsStr>,
    command: &Command,
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
            CaptureError::TooLarge => io::Error::other(format!(
                "{command} printed more than {MAX_QUERY_BYTES} bytes"
            )),
            CaptureError::TimedOut => io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "{command} did not end within {} seconds",
                    QUERY_DEADLINE.as_secs()
                ),
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

    let quiet = lines.is_empty() && captured.errors.trim().is_empty();
    if captured.code == Some(0_i32) || empty_miss(captured.code, quiet) {
        return Ok(lines);
    }

    let ended = captured
        .code
        .map_or_else(|| "a signal".to_owned(), |code| format!("exit code {code}"));

    Err(io::Error::other(format!(
        "{command} ended with {ended}: {}",
        captured.errors.trim()
    )))
}

/// Only exit code 1 with no output and no error text is a miss: pacman
/// `-Qtdq` exits with 1 when there are no orphans. Every other failure
/// keeps its error, so a real problem never reads as an empty answer.
pub fn empty_miss(code: Option<i32>, quiet: bool) -> bool {
    code == Some(1_i32) && quiet
}
