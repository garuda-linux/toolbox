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

pub fn failure(step: StepId, sink: &mut dyn EventSink, message: &str) -> Outcome {
    note(
        step,
        sink,
        message,
        Outcome::Failed {
            cause: Failure::Unknown,
        },
    )
}

pub const fn ended(exit: Exit) -> Outcome {
    Outcome::Failed {
        cause: match exit {
            Exit::Code(code) => Failure::ExitCode { code },
            Exit::Signal(signal) => Failure::Signal { signal },
        },
    }
}
