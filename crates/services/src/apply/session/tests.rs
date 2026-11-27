use toolbox_plan::protocol::{Event, Outcome};
use toolbox_plan::{RootAction, Step, StepId};

use super::step_span;

fn steps() -> Vec<Step<RootAction>> {
    vec![Step {
        id: StepId::new(1),
        action: RootAction::RefreshKeyring,
    }]
}

#[test]
fn a_step_span_lasts_from_start_to_finish() {
    let steps = steps();
    let started = Event::Started {
        step: StepId::new(1),
    };

    let output = Event::Output {
        step: StepId::new(1),
        data: "x".to_owned(),
    };

    let finished = Event::Finished {
        step: StepId::new(1),
        outcome: Outcome::Succeeded,
    };

    let open = step_span(None, Some(&started), &steps);
    assert!(open.is_some());

    let open = step_span(open, Some(&output), &steps);
    assert!(open.is_some());

    let open = step_span(open, None, &steps);
    assert!(open.is_some());

    assert!(step_span(open, Some(&finished), &steps).is_none());
}

#[test]
fn a_step_the_request_does_not_have_opens_no_span() {
    let started = Event::Started {
        step: StepId::new(9),
    };

    assert!(step_span(None, Some(&started), &steps()).is_none());
}

#[test]
fn the_end_of_the_plan_closes_the_span() {
    let started = Event::Started {
        step: StepId::new(1),
    };

    let open = step_span(None, Some(&started), &steps());

    assert!(step_span(open, Some(&Event::Done), &steps()).is_none());
}
