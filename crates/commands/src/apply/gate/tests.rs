use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Outcome};
use toolbox_services::apply::Report;

use super::{Gate, asks_secret};

fn step() -> StepId {
    StepId::new(0)
}

fn output(text: &str) -> Report {
    Report::Step(Event::Output {
        step: step(),
        data: text.to_owned(),
    })
}

#[test]
fn nothing_typed_is_taken_before_a_step_runs() {
    let gate = Gate::default();
    gate.note(&Report::PlanStarted { steps: Vec::new() });
    gate.note(&Report::Step(Event::Accepted));

    assert!(!gate.open());
}

#[test]
fn typed_lines_reach_a_running_step_only() {
    let gate = Gate::default();
    gate.note(&Report::Step(Event::Started { step: step() }));
    assert!(gate.open());

    gate.note(&Report::Step(Event::Finished {
        step: step(),
        outcome: Outcome::Succeeded,
    }));
    assert!(!gate.open());
}

#[test]
fn the_end_of_the_plan_closes_the_gate_for_good() {
    let gate = Gate::default();
    gate.note(&Report::Step(Event::Started { step: step() }));
    gate.note(&Report::PlanFinished { succeeded: true });

    assert!(!gate.open());
    assert!(gate.done());
}

#[test]
fn a_password_question_of_the_step_is_answered_without_echo() {
    let gate = Gate::default();
    gate.note(&Report::Step(Event::Started { step: step() }));
    gate.note(&output("checking\n[sudo] password for "));
    gate.note(&output("nico: "));
    assert!(gate.secret());

    gate.note(&output("\nresolving dependencies\n"));
    assert!(!gate.secret());
}

#[test]
fn only_a_question_for_a_secret_counts() {
    assert!(asks_secret("[sudo] password for nico: "));
    assert!(asks_secret("Enter passphrase for /dev/sda2: "));
    assert!(asks_secret("Passwort: "));
    assert!(!asks_secret(":: Proceed with installation? [Y/n] "));
    assert!(!asks_secret("password changed for nico"));
}
