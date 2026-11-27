use std::collections::VecDeque;

use toolbox_plan::protocol::{Event, Mode, Outcome, Request};
use toolbox_plan::{RootAction, StepId};

use super::{FakeRunner, finished, request, run};

fn gated(mode: Mode, actions: Vec<RootAction>, gates: &[u32]) -> Request {
    Request {
        gates: gates.iter().map(|id| StepId::new(*id)).collect(),
        ..request(mode, actions)
    }
}

fn with_proceeds(proceeds: &[bool]) -> FakeRunner {
    FakeRunner {
        proceeds: VecDeque::from(proceeds.to_vec()),
        ..FakeRunner::default()
    }
}

fn gated_run(request: &Request, runner: &mut FakeRunner) -> (bool, Vec<Event>) {
    let mut events = Vec::new();
    let ok = run(request, runner, &mut events);

    (ok, events)
}

fn refresh_then_clean(mode: Mode, gates: &[u32]) -> Request {
    gated(
        mode,
        vec![RootAction::RefreshKeyring, RootAction::CleanPackageCache],
        gates,
    )
}

fn waited(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::Waiting { step: _ }))
}

#[test]
fn a_gated_step_runs_once_the_app_proceeds() {
    let mut runner = with_proceeds(&[true]);
    let request = refresh_then_clean(Mode::Run, &[2]);

    let (ok, events) = gated_run(&request, &mut runner);

    assert!(ok);
    assert_eq!(
        runner.ran,
        ["pacman-key --refresh-keys", "wait", "paccache -ruk0"]
    );
    assert!(events.contains(&Event::Waiting {
        step: StepId::new(2)
    }));
    assert_eq!(
        finished(&events),
        [
            (StepId::new(1), Outcome::Succeeded),
            (StepId::new(2), Outcome::Succeeded)
        ]
    );
}

#[test]
fn a_stop_at_a_gate_runs_no_further_step() {
    let mut runner = with_proceeds(&[false]);
    let request = gated(
        Mode::Run,
        vec![
            RootAction::RefreshKeyring,
            RootAction::CleanPackageCache,
            RootAction::RefreshKeyring,
        ],
        &[2],
    );

    let (ok, events) = gated_run(&request, &mut runner);

    assert!(!ok);
    assert_eq!(runner.ran, ["pacman-key --refresh-keys", "wait"]);
    assert_eq!(
        finished(&events).last(),
        Some(&(StepId::new(2), Outcome::Cancelled))
    );
    assert_eq!(events.last(), Some(&Event::Done));
}

#[test]
fn a_stop_at_a_gate_keeps_the_cleanups_of_the_steps_before_it() {
    let mut runner = with_proceeds(&[false]);
    let request = gated(
        Mode::Run,
        vec![
            RootAction::SetWifiBackend {
                backend: toolbox_plan::WifiBackend::Iwd,
            },
            RootAction::CleanPackageCache,
        ],
        &[2],
    );

    gated_run(&request, &mut runner);

    assert_eq!(runner.ran.last().map(String::as_str), Some("wait"));
    assert!(
        runner.ran.iter().any(|entry| entry.starts_with("always: ")),
        "{:?}",
        runner.ran
    );
}

#[test]
fn a_gate_after_a_failed_step_is_never_reached() {
    let mut runner = FakeRunner::default();
    runner.outcomes.push_back(Outcome::Failed {
        cause: toolbox_plan::protocol::Failure::ExitCode { code: 1_i32 },
        cleanup_failed: Vec::new(),
    });

    let (ok, events) = gated_run(&refresh_then_clean(Mode::Run, &[2]), &mut runner);

    assert!(!ok);
    assert_eq!(runner.ran, ["pacman-key --refresh-keys"]);
    assert!(!waited(&events), "{events:?}");
}

#[test]
fn a_dry_run_waits_at_its_gate_too() {
    let mut runner = with_proceeds(&[true]);

    let (ok, _events) = gated_run(&refresh_then_clean(Mode::DryRun, &[2]), &mut runner);

    assert!(ok);
    assert_eq!(runner.ran, ["wait"]);
}

#[test]
fn a_request_without_gates_never_waits() {
    let mut runner = FakeRunner::default();

    let (_ok, events) = gated_run(&refresh_then_clean(Mode::Run, &[]), &mut runner);

    assert_eq!(runner.ran, ["pacman-key --refresh-keys", "paccache -ruk0"]);
    assert!(!waited(&events), "{events:?}");
}
