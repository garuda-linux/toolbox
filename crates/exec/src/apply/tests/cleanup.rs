//! A repair whose unmount fails after its main command failed.

use std::collections::VecDeque;

use toolbox_plan::procedure::{Command, Invocation, Program};
use toolbox_plan::protocol::{Event, Failure, Leftover, Outcome};
use toolbox_plan::{RepairMount, StepId};

use super::FakeRunner;
use crate::apply::run_procedure;

fn procedure() -> Vec<Invocation> {
    vec![
        Invocation::Run(Command::new(Program::Mount).args(["/dev/sdb2", "/mnt"])),
        Invocation::Cleanup {
            command: Command::new(Program::Umount).args(["-R", "/mnt"]),
            leftover: Leftover::Mounted {
                place: RepairMount::Root,
            },
        },
    ]
}

fn failed(code: i32) -> Outcome {
    Outcome::Failed {
        cause: Failure::ExitCode { code },
        cleanup_failed: Vec::new(),
    }
}

#[test]
fn the_first_failure_stays_the_cause_and_the_leftover_comes_along() {
    let mut runner = FakeRunner {
        outcomes: VecDeque::from([failed(1_i32), failed(32_i32)]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let outcome = run_procedure(StepId::new(1), &procedure(), &mut runner, &mut events);

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::ExitCode { code: 1_i32 },
            cleanup_failed: vec![Leftover::Mounted {
                place: RepairMount::Root
            }],
        }
    );
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Output { step: _, data } if data.contains("is still mounted")
    )));
}

#[test]
fn a_cleanup_that_failed_alone_fails_the_step() {
    let mut runner = FakeRunner {
        outcomes: VecDeque::from([Outcome::Succeeded, failed(32_i32)]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let outcome = run_procedure(StepId::new(1), &procedure(), &mut runner, &mut events);

    assert!(matches!(
        outcome,
        Outcome::Failed {
            cause: Failure::ExitCode { code: 32_i32 },
            cleanup_failed,
        } if cleanup_failed.len() == 1
    ));
}

#[test]
fn a_cleanup_that_worked_leaves_the_outcome_alone() {
    let mut runner = FakeRunner {
        outcomes: VecDeque::from([failed(1_i32), Outcome::Succeeded]),
        ..FakeRunner::default()
    };

    let outcome = run_procedure(StepId::new(1), &procedure(), &mut runner, &mut Vec::new());

    assert_eq!(outcome, failed(1_i32));
}
