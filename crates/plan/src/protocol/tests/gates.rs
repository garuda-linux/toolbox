use super::{Answer, Event, Mode, PROTOCOL_VERSION, Request, RequestError, decode, encode, step};
use crate::{RootAction, Step, StepId};

fn two_steps() -> Vec<Step<RootAction>> {
    vec![
        step(1, RootAction::RefreshKeyring),
        step(2, RootAction::CleanPackageCache),
    ]
}

fn gated(gates: &[u32]) -> Request {
    Request {
        gates: gates.iter().map(|id| StepId::new(*id)).collect(),
        ..Request::new(Mode::Run, two_steps())
    }
}

#[test]
fn a_gate_on_a_step_the_request_does_not_have_is_refused() {
    assert_eq!(
        gated(&[3]).validate(),
        Err(RequestError::UnknownGate(StepId::new(3)))
    );
}

#[test]
fn a_gate_on_the_first_step_is_refused() {
    assert_eq!(
        gated(&[1]).validate(),
        Err(RequestError::GateOnFirstStep(StepId::new(1)))
    );
}

#[test]
fn a_gate_named_twice_is_refused() {
    assert_eq!(
        gated(&[2, 2]).validate(),
        Err(RequestError::DuplicateGate(StepId::new(2)))
    );
}

#[test]
fn a_gate_on_a_later_step_is_accepted() {
    assert_eq!(gated(&[2]).validate(), Ok(()));
}

#[test]
fn a_gate_naming_an_unknown_step_is_refused_while_the_request_is_read() {
    let line = encode(&gated(&[3])).unwrap();

    assert_eq!(
        Request::parse(&line),
        Err(RequestError::UnknownGate(StepId::new(3)))
    );
}

#[test]
fn gates_come_back_from_the_wire() {
    let request = gated(&[2]);
    let line = encode(&request).unwrap();

    assert_eq!(Request::parse(&line).unwrap(), request);
}

#[test]
fn a_request_without_gates_carries_no_gates_on_the_wire() {
    let request = Request::new(Mode::Run, two_steps());
    let line = encode(&request).unwrap();

    assert!(!line.contains("gates"), "{line}");
    assert_eq!(Request::parse(&line).unwrap(), request);
}

#[test]
fn the_wire_names_of_the_wait_are_fixed() {
    assert_eq!(
        encode(&Event::Waiting {
            step: StepId::new(2)
        })
        .unwrap(),
        "{\"type\":\"waiting\",\"step\":2}\n"
    );
    assert_eq!(
        encode(&Answer::Proceed {
            step: StepId::new(2)
        })
        .unwrap(),
        "{\"type\":\"proceed\",\"step\":2}\n"
    );
    assert_eq!(
        decode::<Answer>("{\"type\":\"proceed\",\"step\":2}").unwrap(),
        Answer::Proceed {
            step: StepId::new(2)
        }
    );
}

#[test]
fn a_request_of_the_previous_protocol_is_refused() {
    let line = encode(&Request::new(Mode::Run, two_steps()))
        .unwrap()
        .replace(&format!("\"version\":{PROTOCOL_VERSION}"), "\"version\":22");

    assert_eq!(
        Request::parse(&line),
        Err(RequestError::VersionMismatch {
            expected: PROTOCOL_VERSION,
            got: 22
        })
    );
}
