//! How a program is started in its terminal and how its end becomes an
//! outcome.

use std::io;
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Instant;

use toolbox_plan::StepId;
use toolbox_plan::procedure::Command;
use toolbox_plan::protocol::{Event, Outcome};
use toolbox_pty::{Chunk, Exit, PtyCommand, Session, TerminalSize};

use super::{Environment, LAST_OUTPUT};
use crate::apply::EventSink;
use crate::report::{ended, note};

pub(super) fn pty_command(
    program: &Path,
    command: &Command,
    size: TerminalSize,
    environment: Environment,
) -> PtyCommand {
    let base = PtyCommand::new(program)
        .args(command.arguments())
        .size(size);

    let base = match environment {
        Environment::Clean => base.env_clear(),
        Environment::Inherited => base,
    };

    environment
        .variables()
        .iter()
        .chain(command.variables())
        .fold(base, |pty, (key, value)| pty.env(key, value))
}

/// The last lines of a failing program say why. They are read until the
/// terminal closes, but a program that left a child holding it open does
/// not hold the step longer than [`LAST_OUTPUT`].
pub(super) fn forward_rest(step: StepId, session: &Session, sink: &mut dyn EventSink) {
    let end = Instant::now() + LAST_OUTPUT;

    loop {
        let left = end.saturating_duration_since(Instant::now());

        match session.output().recv_timeout(left) {
            Ok(Chunk::Text(data)) => sink.send(Event::Output { step, data }),
            Ok(Chunk::Closed) | Err(RecvTimeoutError::Disconnected) => return,
            Err(RecvTimeoutError::Timeout) => {
                tracing::warn!("the terminal stayed open after the program ended");
                return;
            }
        }
    }
}

pub(super) fn report(result: io::Result<()>, what: &str) {
    if let Err(error) = result {
        tracing::warn!("cannot {what} the running program: {error}");
    }
}

/// A cancel read in the same pass as a clean exit came too late: the
/// program finished its work, `pacman -S` may have committed.
pub(super) fn after(
    step: StepId,
    exit: Exit,
    cancelled: bool,
    sink: &mut dyn EventSink,
) -> Outcome {
    match (exit, cancelled) {
        (Exit::Code(0), true) => note(
            step,
            sink,
            "the cancel came after the program had finished",
            Outcome::Succeeded,
        ),
        (_, true) => Outcome::Cancelled,
        (Exit::Code(0), false) => Outcome::Succeeded,
        (exit, false) => ended(exit),
    }
}
