//! Messages the runner and the builtins show in the Console beside a
//! step's outcome.

use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Failure, Outcome};
use toolbox_pty::Exit;

use crate::apply::EventSink;

pub fn note(step: StepId, sink: &mut dyn EventSink, message: &str, outcome: Outcome) -> Outcome {
    sink.send(Event::Output {
        step,
        data: format!("{message}\r\n"),
    });

    outcome
}

/// The Console is a terminal: a bare line feed would not return the cursor to the first column.
pub fn output(step: StepId, sink: &mut dyn EventSink, text: &str) {
    if !text.is_empty() {
        sink.send(Event::Output {
            step,
            data: text.replace("\r\n", "\n").replace('\n', "\r\n"),
        });
    }
}

pub fn failure(step: StepId, sink: &mut dyn EventSink, message: &str) -> Outcome {
    note(
        step,
        sink,
        message,
        Outcome::Failed {
            cause: Failure::Unknown,
            cleanup_failed: Vec::new(),
        },
    )
}

pub const fn ended(exit: Exit) -> Outcome {
    Outcome::Failed {
        cause: match exit {
            Exit::Code(code) => Failure::ExitCode { code },
            Exit::Signal(signal) => Failure::Signal { signal },
        },
        cleanup_failed: Vec::new(),
    }
}
