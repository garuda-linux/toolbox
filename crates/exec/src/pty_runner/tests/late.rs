//! A cancel and the program's end read in one pass.

use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Failure, Outcome};
use toolbox_pty::Exit;

use crate::pty_runner::after;

fn outcome(exit: Exit, cancelled: bool) -> (Outcome, Vec<Event>) {
    let mut events = Vec::new();
    let outcome = after(StepId::new(1), exit, cancelled, &mut events);

    (outcome, events)
}

#[test]
fn a_program_that_finished_before_the_cancel_succeeded() {
    let (outcome, events) = outcome(Exit::Code(0), true);

    assert_eq!(outcome, Outcome::Succeeded);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Output { step: _, data } if data.contains("after the program had finished")
    )));
}

#[test]
fn a_program_that_failed_while_cancelled_is_cancelled() {
    assert_eq!(outcome(Exit::Code(1), true).0, Outcome::Cancelled);
}

#[test]
fn without_a_cancel_the_exit_decides_and_nothing_is_said() {
    let (outcome_ok, said) = outcome(Exit::Code(0), false);

    assert_eq!(outcome_ok, Outcome::Succeeded);
    assert_eq!(said, Vec::new());
    assert!(matches!(
        outcome(Exit::Code(2), false).0,
        Outcome::Failed {
            cause: Failure::ExitCode { code: 2_i32 },
            cleanup_failed: _
        }
    ));
}
