//! Read-only programs the adapters ask.

#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::process::{Command, ExitStatus};
use std::sync::Arc;
use std::time::{Duration, Instant};

use toolbox_exec::{CaptureError, Environment, Policy, run_captured};
use toolbox_pty::strip_ansi;

use crate::error::{ProbeError, from_io};

/// How long a program may take. A hung `checkupdates` would otherwise
/// hold its probe for the whole session.
pub const DEADLINE: Duration = Duration::from_secs(120);

/// How long a program past its deadline may clean up after SIGTERM.
const STOP_GRACE: Duration = Duration::from_secs(5);

/// English error text in the log, whatever the user's locale.
const ENGLISH: [(&str, &str); 2] = [("LANG", "C"), ("LC_ALL", "C")];

/// A program that ran to its end, whatever its exit status. For programs
/// whose exit code is an answer, like `checkupdates` (2: no updates).
pub struct Finished {
    /// The program and its arguments, for messages.
    pub command_line: String,
    pub status: ExitStatus,
    /// Lossy UTF-8.
    pub stdout: String,
    /// Lossy UTF-8.
    pub stderr: String,
}

impl Finished {
    /// The failure with the first line of stderr. Escape sequences are
    /// stripped, as progress bars and colours are no text for a Label.
    pub fn failure(&self) -> ProbeError {
        let reason = strip_ansi(self.stderr.lines().next().unwrap_or_default().trim());

        ProbeError::Failed(format!("{} {}: {reason}", self.command_line, self.status))
    }
}

/// What `program` printed on success. A program that cannot be found is
/// unavailable, one that exits with an error or runs past [`DEADLINE`]
/// has failed.
pub fn output(program: &OsStr, args: &[&str]) -> Result<String, ProbeError> {
    let finished = run(program, args)?;

    if finished.status.success() {
        Ok(finished.stdout)
    } else {
        Err(finished.failure())
    }
}

/// [`run_within`] the usual [`DEADLINE`].
pub fn run(program: &OsStr, args: &[&str]) -> Result<Finished, ProbeError> {
    run_within(program, args, DEADLINE)
}

/// Runs `program` to its end. A program that cannot be started is an
/// error (missing means unavailable, anything else failed), and so is one
/// that runs past `deadline`: it and everything it started get SIGTERM,
/// then SIGKILL after a grace.
pub fn run_within(
    program: &OsStr,
    args: &[&str],
    deadline: Duration,
) -> Result<Finished, ProbeError> {
    let started = Instant::now();
    let command_line = format!("{} {}", program.display(), args.join(" "));
    let mut command = Command::new(program);

    command.args(args);

    let policy = Policy::whole().within(deadline).term_first(STOP_GRACE);
    let exited = run_captured(
        &mut command,
        Environment::Inherited,
        &ENGLISH,
        &policy,
        &Arc::default(),
    )
    .map_err(|error| match error {
        CaptureError::Missing => ProbeError::Unavailable,
        CaptureError::Failed(error) => from_io(&command_line, error),
        CaptureError::TimedOut => ProbeError::Failed(format!(
            "{command_line}: no answer within {} s, stopped",
            deadline.as_secs_f32()
        )),
        CaptureError::Cancelled | CaptureError::TooLarge => {
            ProbeError::Failed(format!("{command_line}: {error}"))
        }
    })?;

    tracing::debug!(
        "{command_line}: {} after {} ms",
        exited.status,
        started.elapsed().as_millis()
    );

    Ok(Finished {
        command_line,
        status: exited.status,
        stdout: text(&exited.output),
        stderr: text(&exited.errors),
    })
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
