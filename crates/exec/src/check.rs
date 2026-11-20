//! Runs a checker that reads a configuration text on its standard input, as
//! `visudo -c -f -` does before a merge. The text is no secret, so the
//! checker's words reach the Console as they are. A secret is fed apart, in
//! `crate::feed`.

#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use toolbox_plan::procedure::Command;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{ConfigText, StepId};

use crate::apply::EventSink;
use crate::capture::{self, CaptureError, Exited, MAX_ERROR_BYTES, Policy};
use crate::feed::FEED_DEADLINE;
use crate::pty_runner::Environment;
use crate::report::{ended, failure, output};

/// A hung check would hold the helper, so it gets a feed's deadline.
pub const CHECK_DEADLINE: Duration = FEED_DEADLINE;

/// `cancel` stops it, as it stops a query.
///
/// # Errors
///
/// As [`capture::run_captured`].
pub fn check(
    command: &Command,
    environment: Environment,
    text: &ConfigText,
    cancel: &Arc<AtomicBool>,
) -> Result<Exited, CaptureError> {
    run(
        crate::located::program(command.program(), environment),
        command,
        environment,
        &policy(text, CHECK_DEADLINE),
        cancel,
    )
}

pub fn report(
    step: StepId,
    command: &Command,
    deadline: Duration,
    result: Result<Exited, CaptureError>,
    sink: &mut dyn EventSink,
) -> Outcome {
    let program = command.program();
    let exited = match result {
        Ok(exited) => exited,
        Err(CaptureError::Cancelled) => return Outcome::Cancelled,
        Err(CaptureError::TimedOut) => {
            return failure(
                step,
                sink,
                &format!(
                    "{}: it did not end within {} seconds and was stopped",
                    program.name(),
                    deadline.as_secs()
                ),
            );
        }
        Err(CaptureError::Missing) => {
            return failure(
                step,
                sink,
                &format!("cannot run {command}: {} is not installed", program.path()),
            );
        }
        Err(error @ (CaptureError::Failed(_) | CaptureError::TooLarge)) => {
            return failure(step, sink, &format!("cannot run {command}: {error}"));
        }
    };

    let printed = [exited.output.as_slice(), exited.errors.as_slice()].concat();
    output(step, sink, &String::from_utf8_lossy(&printed));

    if exited.status.success() {
        Outcome::Succeeded
    } else {
        ended(exited.status.into())
    }
}

/// The checker's words are short, a flood of them is cut.
fn policy(text: &ConfigText, deadline: Duration) -> Policy<'_> {
    Policy::tail(MAX_ERROR_BYTES)
        .within(deadline)
        .fed(text.as_str().as_bytes())
        .written_apart()
}

/// `program` is a parameter so tests can run a stand-in.
fn run(
    program: impl AsRef<OsStr>,
    command: &Command,
    environment: Environment,
    policy: &Policy<'_>,
    cancel: &Arc<AtomicBool>,
) -> Result<Exited, CaptureError> {
    let mut process = std::process::Command::new(program);

    process.args(command.arguments());
    capture::run_captured(
        &mut process,
        environment,
        command.variables(),
        policy,
        cancel,
    )
}
