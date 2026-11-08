//! Running a program that takes a secret on its standard input. No
//! pseudo-terminal here: a terminal echoes what is written to it, which
//! would send the secret back as output. The program gets plain pipes,
//! and what it prints is scrubbed before it becomes an event.

#[cfg(test)]
mod tests;

use std::process::Output;
use std::sync::Arc;
use std::time::Duration;
use std::{fmt, io};

use toolbox_plan::procedure::Command;
use toolbox_plan::protocol::{Event, Outcome};
use toolbox_plan::{SecretInput, StepId};

use crate::apply::EventSink;
use crate::capture::{self, CaptureError, Policy};
use crate::pty_runner::Environment;
use crate::report::{ended, failure};

/// How long a fed program may take. chpasswd and `cryptsetup open` end
/// within seconds. One that waits for a device or a second prompt would
/// hold the helper, which no cancel reaches here.
pub const FEED_DEADLINE: Duration = Duration::from_secs(120);

#[derive(Debug)]
enum FeedError {
    Failed(io::Error),
    /// It ran longer than this and was killed.
    Late(Duration),
}

impl fmt::Display for FeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(error) => write!(f, "{error}"),
            Self::Late(deadline) => write!(
                f,
                "it did not end within {} seconds and was stopped",
                deadline.as_secs()
            ),
        }
    }
}

impl std::error::Error for FeedError {}

/// Runs `command`, writes `input` to it and closes the pipe. A cancel does
/// not reach it: these programs finish in a moment, and a half-set
/// password is worse than a late stop. Only [`FEED_DEADLINE`] ends one
/// that hangs.
pub fn feed(
    step: StepId,
    command: &Command,
    environment: Environment,
    input: &SecretInput,
    sink: &mut dyn EventSink,
) -> Outcome {
    let result = run(
        crate::located::program(command.program()),
        command,
        environment,
        input,
        FEED_DEADLINE,
    );

    report(step, command, result, input, sink)
}

/// Turns what the program printed and how it ended into events, with the
/// secret taken out of the text.
fn report(
    step: StepId,
    command: &Command,
    result: Result<Output, FeedError>,
    input: &SecretInput,
    sink: &mut dyn EventSink,
) -> Outcome {
    let output = match result {
        Ok(output) => output,
        Err(FeedError::Failed(error)) => {
            return failure(step, sink, &format!("cannot run {command}: {error}"));
        }
        // Without what it printed: nothing scrubbed it.
        Err(late @ FeedError::Late(_)) => {
            return failure(step, sink, &format!("{}: {late}", command.program().name()));
        }
    };

    let printed = [output.stdout.as_slice(), output.stderr.as_slice()].concat();
    let text = input.scrub(&String::from_utf8_lossy(&printed));

    if !text.is_empty() {
        // The Console is a terminal: a bare line feed would not return
        // the cursor to the first column.
        sink.send(Event::Output {
            step,
            data: text.replace("\r\n", "\n").replace('\n', "\r\n"),
        });
    }

    if output.status.success() {
        Outcome::Succeeded
    } else {
        ended(output.status.into())
    }
}

/// `program` and `deadline` are parameters so tests can run a stand-in.
/// No cancel reaches it, and the whole output is kept: it is scrubbed
/// before anyone sees it.
fn run(
    program: impl AsRef<std::ffi::OsStr>,
    command: &Command,
    environment: Environment,
    input: &SecretInput,
    deadline: Duration,
) -> Result<Output, FeedError> {
    let mut process = std::process::Command::new(program);

    process.args(command.arguments());

    let policy = Policy::whole().within(deadline).fed(input.expose());

    match capture::run_captured(
        &mut process,
        environment,
        command.variables(),
        &policy,
        &Arc::default(),
    ) {
        Ok(exited) => Ok(Output {
            status: exited.status,
            stdout: exited.output,
            stderr: exited.errors,
        }),
        Err(CaptureError::TimedOut) => Err(FeedError::Late(deadline)),
        Err(
            other @ (CaptureError::Failed(_)
            | CaptureError::Missing
            | CaptureError::Cancelled
            | CaptureError::TooLarge),
        ) => Err(FeedError::Failed(other.into())),
    }
}
