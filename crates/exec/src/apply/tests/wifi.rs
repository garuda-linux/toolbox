//! A step with an always part: the Wi-Fi backend switch starts
//! NetworkManager again whatever happened before.

use std::collections::VecDeque;

use toolbox_plan::protocol::{Event, Failure, Mode, Outcome};
use toolbox_plan::{RootAction, StepId, WifiBackend};

use super::{FakeRunner, finished, request, run};

pub(super) fn wifi(runner: &mut FakeRunner) -> (bool, Vec<Event>) {
    let mut events = Vec::new();
    let ok = run(
        &request(
            Mode::Run,
            vec![
                RootAction::SetWifiBackend {
                    backend: WifiBackend::Iwd,
                },
                RootAction::RefreshKeyring,
            ],
        ),
        runner,
        &mut events,
    );

    (ok, events)
}

#[test]
fn a_failure_inside_a_step_still_runs_its_always_part() {
    let failed = Outcome::Failed {
        cause: Failure::ExitCode { code: 5_i32 },
        cleanup_failed: Vec::new(),
    };

    let mut runner = FakeRunner {
        outcomes: VecDeque::from([Outcome::Succeeded, failed.clone()]),
        ..FakeRunner::default()
    };

    let (ok, events) = wifi(&mut runner);

    assert!(!ok);
    assert_eq!(
        runner.ran,
        [
            "systemctl stop -- NetworkManager.service",
            "systemctl disable --now -- wpa_supplicant.service",
            "always: systemctl start -- NetworkManager.service"
        ]
    );
    assert_eq!(finished(&events), [(StepId::new(1), failed)]);
}

#[test]
fn a_cancel_inside_a_step_still_runs_its_always_part() {
    let mut runner = FakeRunner {
        outcomes: VecDeque::from([Outcome::Cancelled]),
        ..FakeRunner::default()
    };

    let (ok, events) = wifi(&mut runner);

    assert!(!ok);
    assert_eq!(
        runner.ran,
        [
            "systemctl stop -- NetworkManager.service",
            "always: systemctl start -- NetworkManager.service"
        ]
    );
    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Cancelled)]);
}

#[test]
fn a_failing_always_part_fails_the_step() {
    let failed = Outcome::Failed {
        cause: Failure::ExitCode { code: 1_i32 },
        cleanup_failed: Vec::new(),
    };

    let mut runner = FakeRunner {
        outcomes: VecDeque::from([
            Outcome::Succeeded,
            Outcome::Succeeded,
            Outcome::Succeeded,
            Outcome::Succeeded,
            failed.clone(),
        ]),
        ..FakeRunner::default()
    };

    let (ok, events) = wifi(&mut runner);

    assert!(!ok);
    assert_eq!(finished(&events), [(StepId::new(1), failed)]);
}

#[test]
fn a_step_with_an_always_part_that_succeeds_goes_on() {
    let mut runner = FakeRunner::default();

    let (ok, events) = wifi(&mut runner);

    assert!(ok);
    assert_eq!(runner.ran.len(), 7, "{:?}", runner.ran);
    assert_eq!(
        finished(&events),
        [
            (StepId::new(1), Outcome::Succeeded),
            (StepId::new(2), Outcome::Succeeded)
        ]
    );
}
