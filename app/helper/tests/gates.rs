//! The gates of an apply request, through the helper binary in dry-run mode:
//! nothing runs, so no root is needed.

mod common;

use common::{TestResult, events, helper, outcomes, request};
use toolbox_plan::protocol::{Event, Mode, Outcome, Request, encode};
use toolbox_plan::{RootAction, Step, StepId};

fn gated_request(gates: &[u32]) -> Result<String, Box<dyn std::error::Error>> {
    let steps = [RootAction::RefreshKeyring, RootAction::CleanPackageCache]
        .into_iter()
        .zip(1_u32..)
        .map(|(action, id)| Step {
            id: StepId::new(id),
            action,
        })
        .collect();

    let request = Request {
        gates: gates.iter().map(|id| StepId::new(*id)).collect(),
        ..Request::new(Mode::DryRun, steps)
    };

    Ok(encode(&request)?)
}

#[test]
fn the_helper_waits_at_a_gate_and_goes_on_when_the_app_proceeds() -> TestResult {
    let run = helper(&["apply"], &gated_request(&[2])?)?;

    let order: Vec<Event> = events(&run)?
        .into_iter()
        .filter(|event| {
            matches!(
                event,
                Event::Started { step: _ } | Event::Waiting { step: _ }
            )
        })
        .collect();

    assert_eq!(
        order,
        [
            Event::Started {
                step: StepId::new(1)
            },
            Event::Waiting {
                step: StepId::new(2)
            },
            Event::Started {
                step: StepId::new(2)
            },
        ]
    );
    assert_eq!(outcomes(&run)?, [Outcome::Skipped, Outcome::Skipped]);
    assert!(run.status.success());

    Ok(())
}

#[test]
fn a_gate_on_an_unknown_step_refuses_the_request_before_any_step() -> TestResult {
    let run = helper(&["apply"], &gated_request(&[9])?)?;
    let events = events(&run)?;

    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::Rejected { reason: _ })),
        "{events:?}"
    );
    assert!(
        !events.iter().any(|event| matches!(
            event,
            Event::Started { step: _ } | Event::Waiting { step: _ }
        )),
        "{events:?}"
    );
    assert!(!run.status.success());
    assert!(
        run.stderr.contains("the gate names step 9"),
        "{}",
        run.stderr
    );

    Ok(())
}

#[test]
fn a_request_without_gates_never_waits() -> TestResult {
    let run = helper(
        &["apply"],
        &request(Mode::DryRun, vec![RootAction::RefreshKeyring])?,
    )?;

    assert!(
        !events(&run)?
            .iter()
            .any(|event| matches!(event, Event::Waiting { step: _ })),
        "{}",
        run.stdout
    );
    assert!(run.status.success());

    Ok(())
}
