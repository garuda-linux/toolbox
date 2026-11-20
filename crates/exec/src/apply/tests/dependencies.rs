//! Only unmet names go to pacman, so installed packages keep their install reason.

use std::collections::VecDeque;

use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{PackageName, RootAction, StepId, TargetList};

use super::{FakeRunner, finished, request, run};

const TEST: &str = "deptest: pacman -T -- cmake qt6-tools";

fn install() -> RootAction {
    RootAction::InstallDependencies {
        packages: TargetList::new(
            ["qt6-tools", "cmake"].map(|name| PackageName::new(name).unwrap()),
        )
        .unwrap(),
    }
}

fn answering(lines: &[&str]) -> FakeRunner {
    FakeRunner {
        queries: VecDeque::from([lines.iter().map(|&line| line.to_owned()).collect()]),
        ..FakeRunner::default()
    }
}

fn said(events: &[Event], text: &str) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::Output { step: _, data } if data.contains(text)))
}

#[test]
fn only_the_unmet_names_are_installed() {
    let mut runner = answering(&["qt6-tools"]);
    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![install()]),
        &mut runner,
        &mut events,
    );

    assert!(ok);
    assert_eq!(
        runner.ran,
        [TEST, "pacman -S --needed --asdeps --noconfirm -- qt6-tools"]
    );
    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Succeeded)]);
}

#[test]
fn nothing_unmet_is_nothing_to_do() {
    let mut runner = answering(&[]);
    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![install()]),
        &mut runner,
        &mut events,
    );

    assert!(ok);
    assert_eq!(runner.ran, [TEST]);
    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Skipped)]);
}

#[test]
fn a_name_that_was_not_asked_fails_the_step() {
    let mut runner = answering(&["cmake", "linux-lts"]);
    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![install()]),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(runner.ran, [TEST]);
    assert!(said(&events, "linux-lts"), "{events:?}");
}

#[test]
fn a_line_that_is_no_package_name_fails_the_step() {
    let mut runner = answering(&["--overwrite=*"]);
    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![install()]),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(runner.ran, [TEST]);
    assert!(said(&events, "--overwrite"), "{events:?}");
}
