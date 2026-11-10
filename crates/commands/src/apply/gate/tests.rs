use toolbox_plan::StepId;
use toolbox_plan::protocol::{Event, Outcome};
use toolbox_services::apply::Report;

use super::{Gate, asks_secret};

/// `sudo -k true` as `guest` in `script(1)`, with sudo's lecture, a wrong
/// password and its second question.
const SUDO: &str = include_str!("fixtures/sudo-prompt");
/// `cryptsetup open` in `script(1)`, with a wrong passphrase.
const CRYPTSETUP: &str = include_str!("fixtures/cryptsetup-prompt");
/// pacman asking for a `java-runtime` provider, then whether to install.
const PACMAN: &str = toolbox_pty::testing::PACMAN_PROVIDER_TXT;

const SUDO_QUESTION: &str = "[sudo] password for guest: ";

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

    let asked = SUDO.find(SUDO_QUESTION).unwrap() + SUDO_QUESTION.len();
    let (lecture, rest) = SUDO.split_at(asked);
    let (before, name) = lecture.split_at(lecture.len() - "guest: ".len());

    gate.note(&output(before));
    gate.note(&output(name));
    assert!(gate.secret());

    let refused = rest.find("doofus\r\n").unwrap() + "doofus\r\n".len();
    let (typed, again) = rest.split_at(refused);

    gate.note(&output(typed));
    assert!(!gate.secret());

    gate.note(&output(again));
    assert_eq!(again, SUDO_QUESTION);
    assert!(gate.secret());
}

#[test]
fn only_a_question_for_a_secret_counts() {
    let last_line = |text: &'static str| text.rsplit('\n').next().unwrap();
    let mut cryptsetup = CRYPTSETUP.lines();
    let (asked, refused) = (cryptsetup.next().unwrap(), cryptsetup.next().unwrap());

    // sudo in German asks with the same shape.
    let german = SUDO_QUESTION.replace("password for", "Passwort für");

    assert!(asks_secret(last_line(SUDO)));
    assert!(asks_secret(asked));
    assert!(asks_secret(&german));
    assert!(!asks_secret(refused));
    assert!(!asks_secret(last_line(PACMAN)));

    let provider = PACMAN
        .split('\n')
        .find(|line| line.starts_with("Enter a number"))
        .unwrap();

    assert!(!asks_secret(provider));
}
