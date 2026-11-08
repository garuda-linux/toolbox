mod panics;
mod secrets;

mod extra;
mod finish;
mod span;

use std::collections::VecDeque;
use std::io;

use toolbox_plan::procedure::{Builtin, Command};
use toolbox_plan::protocol::{Event, Failure, Mode, Outcome, Request};
use toolbox_plan::{RootAction, SecretInput, Step, StepId, WifiBackend};

use super::{EventSink, Runner, apply};

#[derive(Default)]
struct FakeRunner {
    ran: Vec<String>,
    outcomes: VecDeque<Outcome>,
    query_output: Vec<String>,
    fed: Vec<Vec<u8>>,
    /// Part of a command at which `run` panics, as a bug in a runner would.
    panics_at: Option<&'static str>,
    finish_after: Option<usize>,
}

impl Runner for FakeRunner {
    fn finish_requested(&mut self) -> bool {
        self.finish_after
            .is_some_and(|after| self.ran.len() >= after)
    }

    fn run(&mut self, _step: StepId, command: &Command, _sink: &mut dyn EventSink) -> Outcome {
        self.ran.push(command.to_string());

        assert!(
            !self
                .panics_at
                .is_some_and(|part| command.to_string().contains(part)),
            "the runner broke"
        );

        self.outcomes.pop_front().unwrap_or(Outcome::Succeeded)
    }

    fn run_always(
        &mut self,
        _step: StepId,
        command: &Command,
        _sink: &mut dyn EventSink,
    ) -> Outcome {
        self.ran.push(format!("always: {command}"));

        assert!(
            !self
                .panics_at
                .is_some_and(|part| command.to_string().contains(part)),
            "the runner broke"
        );

        self.outcomes.pop_front().unwrap_or(Outcome::Succeeded)
    }

    fn query(&mut self, command: &Command) -> io::Result<Vec<String>> {
        self.ran.push(format!("query: {command}"));
        Ok(self.query_output.clone())
    }

    fn builtin(&mut self, _step: StepId, builtin: &Builtin, _sink: &mut dyn EventSink) -> Outcome {
        self.ran.push(format!("builtin: {builtin:?}"));
        Outcome::Succeeded
    }

    fn feed(
        &mut self,
        _step: StepId,
        command: &Command,
        input: &SecretInput,
        _sink: &mut dyn EventSink,
    ) -> Outcome {
        self.ran.push(format!("feed: {command}"));
        self.fed.push(input.expose().to_vec());
        self.outcomes.pop_front().unwrap_or(Outcome::Succeeded)
    }
}

fn run(request: &Request, runner: &mut FakeRunner, events: &mut Vec<Event>) -> bool {
    apply(request.mode, &request.steps, runner, events)
}

fn request(mode: Mode, actions: Vec<RootAction>) -> Request {
    let steps = (1_u32..)
        .zip(actions)
        .map(|(id, action)| Step {
            id: StepId::new(id),
            action,
        })
        .collect();

    Request::new(mode, steps)
}

fn finished(events: &[Event]) -> Vec<(StepId, Outcome)> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Finished { step, outcome } => Some((*step, *outcome)),
            Event::Accepted
            | Event::Rejected { reason: _ }
            | Event::Started { step: _ }
            | Event::Output { step: _, data: _ }
            | Event::Done => None,
        })
        .collect()
}

#[test]
fn every_step_runs_in_order_and_reports() {
    let mut runner = FakeRunner::default();
    let mut events = Vec::new();

    let ok = run(
        &request(
            Mode::Run,
            vec![RootAction::RefreshKeyring, RootAction::CleanPackageCache],
        ),
        &mut runner,
        &mut events,
    );

    assert!(ok);
    assert_eq!(runner.ran, ["pacman-key --refresh-keys", "paccache -ruk0"]);
    assert_eq!(events.first(), Some(&Event::Accepted));
    assert_eq!(events.last(), Some(&Event::Done));
    assert_eq!(
        finished(&events),
        [
            (StepId::new(1), Outcome::Succeeded),
            (StepId::new(2), Outcome::Succeeded)
        ]
    );
}

#[test]
fn a_failed_step_stops_the_rest() {
    let mut runner = FakeRunner {
        outcomes: VecDeque::from([Outcome::Failed {
            cause: Failure::ExitCode { code: 1_i32 },
        }]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(
            Mode::Run,
            vec![RootAction::RefreshKeyring, RootAction::CleanPackageCache],
        ),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(runner.ran, ["pacman-key --refresh-keys"]);
    assert_eq!(finished(&events).len(), 1);
    assert_eq!(events.last(), Some(&Event::Done));
}

#[test]
fn a_cancelled_step_stops_the_rest() {
    let mut runner = FakeRunner {
        outcomes: VecDeque::from([Outcome::Cancelled]),
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(
            Mode::Run,
            vec![RootAction::RefreshKeyring, RootAction::CleanPackageCache],
        ),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Cancelled)]);
}

#[test]
fn a_dry_run_reports_every_step_as_skipped_and_runs_nothing() {
    let mut runner = FakeRunner::default();
    let mut events = Vec::new();

    let ok = run(
        &request(
            Mode::DryRun,
            vec![RootAction::RefreshKeyring, RootAction::RemoveOrphans],
        ),
        &mut runner,
        &mut events,
    );

    assert!(ok);
    assert!(runner.ran.is_empty(), "{:?}", runner.ran);
    assert_eq!(
        finished(&events),
        [
            (StepId::new(1), Outcome::Skipped),
            (StepId::new(2), Outcome::Skipped)
        ]
    );
}

#[test]
fn query_targets_are_appended_to_the_command() {
    let mut runner = FakeRunner {
        query_output: vec!["libfoo".to_owned(), "python-bar".to_owned()],
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    run(
        &request(Mode::Run, vec![RootAction::RemoveOrphans]),
        &mut runner,
        &mut events,
    );

    assert_eq!(
        runner.ran,
        ["query: pacman -Qtdq", "pacman -Rns -- libfoo python-bar"]
    );
}

#[test]
fn an_empty_query_skips_the_command() {
    let mut runner = FakeRunner::default();
    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![RootAction::RemoveOrphans]),
        &mut runner,
        &mut events,
    );

    assert!(ok);
    assert_eq!(runner.ran, ["query: pacman -Qtdq"]);
    assert_eq!(finished(&events), [(StepId::new(1), Outcome::Skipped)]);
}

#[test]
fn query_output_that_is_not_a_package_name_fails_the_step() {
    let mut runner = FakeRunner {
        query_output: vec!["--overwrite=*".to_owned()],
        ..FakeRunner::default()
    };

    let mut events = Vec::new();

    let ok = run(
        &request(Mode::Run, vec![RootAction::RemoveOrphans]),
        &mut runner,
        &mut events,
    );

    assert!(!ok);
    assert_eq!(runner.ran, ["query: pacman -Qtdq"]);
    assert!(
        events.iter().any(
            |event| matches!(event, Event::Output { step: _, data } if data.contains("--overwrite"))
        ),
        "{events:?}"
    );
}

#[test]
fn builtins_go_to_the_runner() {
    let mut runner = FakeRunner::default();
    let mut events = Vec::new();

    run(
        &request(Mode::Run, vec![RootAction::RemoveDatabaseLock]),
        &mut runner,
        &mut events,
    );

    assert_eq!(runner.ran, ["builtin: RemoveDatabaseLock"]);
}

fn wifi(runner: &mut FakeRunner) -> (bool, Vec<Event>) {
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
    };

    let mut runner = FakeRunner {
        outcomes: VecDeque::from([Outcome::Succeeded, failed]),
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
    };

    let mut runner = FakeRunner {
        outcomes: VecDeque::from([
            Outcome::Succeeded,
            Outcome::Succeeded,
            Outcome::Succeeded,
            Outcome::Succeeded,
            failed,
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
