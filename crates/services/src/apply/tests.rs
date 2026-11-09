mod budget;
mod finish;
mod secrets;
mod size;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tokio::process::Command as Process;
use tokio::sync::mpsc::unbounded_channel;
use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::{Event, Failure, Mode, Outcome, encode};
use toolbox_plan::{Plan, PlannedAction, RootAction, StepId, UserAction};
use toolbox_system::pkexec::Refusal;

use super::{ApplyCommand, Launcher, Problem, Report, Reporter, serve};

struct FakeLauncher {
    script: String,
    mode: Mode,
}

impl Launcher for FakeLauncher {
    fn command(&self) -> Process {
        let mut command = Process::new("sh");
        command.arg("-c").arg(&self.script);
        command
    }

    fn mode(&self) -> Mode {
        self.mode
    }
}

struct MissingHelper;

impl Launcher for MissingHelper {
    fn command(&self) -> Process {
        Process::new("/nonexistent/garuda-toolbox-helper")
    }

    fn mode(&self) -> Mode {
        Mode::Run
    }
}

fn events_script(events: &[Event]) -> String {
    let lines: String = events.iter().map(|event| encode(event).unwrap()).collect();

    format!("read request; printf '%s' '{lines}'")
}

fn recording_script(record: &Path, lines: usize) -> String {
    let reads = "IFS= read -r line; printf \"%s\\n\" \"$line\" >> \"$0\"; ".repeat(lines);

    format!(
        "sh -c '{reads}' {}; printf '%s' '{}'",
        record.display(),
        encode(&Event::Done).unwrap()
    )
}

fn root_plan() -> Plan {
    Plan::root(RootAction::RefreshKeyring)
}

async fn run_after(
    commands: Vec<ApplyCommand>,
    plan: Plan,
    launcher: impl Launcher + 'static,
) -> Vec<Report> {
    let (sender, receiver) = unbounded_channel();
    let (reports, report_queue) = crate::waking::channel(crate::waking::Wake::none());
    let reporter = Reporter(reports);

    for command in commands {
        sender.send(command).unwrap();
    }

    sender.send(ApplyCommand::Run(plan)).unwrap();
    drop(sender);
    tokio::time::timeout(
        Duration::from_secs(20),
        serve(receiver, Arc::new(launcher), reporter),
    )
    .await
    .unwrap();

    report_queue.try_iter().collect()
}

async fn run(plan: Plan, launcher: impl Launcher + 'static) -> Vec<Report> {
    run_after(Vec::new(), plan, launcher).await
}

fn started() -> Report {
    Report::PlanStarted {
        steps: vec![(StepId::new(1), "pacman-key --refresh-keys".to_owned())],
    }
}

#[tokio::test]
async fn a_root_segment_reports_the_helpers_events() {
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Started { step },
        Event::Finished {
            step,
            outcome: Outcome::Succeeded,
        },
        Event::Done,
    ];

    let launcher = FakeLauncher {
        script: events_script(&events),
        mode: Mode::Run,
    };

    let reports = run(root_plan(), launcher).await;

    let mut expected = vec![started()];
    expected.extend(events.into_iter().map(Report::Step));
    expected.push(Report::PlanFinished { succeeded: true });
    assert_eq!(reports, expected);
}

#[tokio::test]
async fn a_failed_root_step_fails_the_plan() {
    let step = StepId::new(1);
    let events = [
        Event::Accepted,
        Event::Finished {
            step,
            outcome: Outcome::Failed {
                cause: Failure::ExitCode { code: 1_i32 },
            },
        },
        Event::Done,
    ];

    let launcher = FakeLauncher {
        script: format!("{}; exit 1", events_script(&events)),
        mode: Mode::Run,
    };

    let reports = run(root_plan(), launcher).await;

    assert_eq!(
        reports.last(),
        Some(&Report::PlanFinished { succeeded: false })
    );
}

#[tokio::test]
async fn a_helper_that_dies_mid_plan_is_reported_with_its_end() {
    let events = [
        Event::Accepted,
        Event::Started {
            step: StepId::new(1),
        },
    ];

    let launcher = FakeLauncher {
        script: format!("{}; kill -9 $$", events_script(&events)),
        mode: Mode::Run,
    };

    let reports = run(root_plan(), launcher).await;

    assert!(
        reports.iter().any(|report| matches!(
            report,
            Report::Problem(Problem::HelperFailed { kind: HelperFailure::Ended, detail })
                if detail.contains("signal")
        )),
        "{reports:?}"
    );
}

#[tokio::test]
async fn pkexec_refusals_are_told_apart_by_exit_code_and_words() {
    for (script, refusal) in [
        ("exit 126", Refusal::Dismissed),
        (
            "echo 'Error executing command as another user: Not authorized' >&2; exit 127",
            Refusal::Denied,
        ),
        (
            "echo 'Error executing command as another user: No authentication agent found.' >&2; exit 127",
            Refusal::NoAgent,
        ),
    ] {
        let launcher = FakeLauncher {
            script: script.to_owned(),
            mode: Mode::Run,
        };

        let reports = run(root_plan(), launcher).await;

        assert_eq!(
            reports,
            [
                started(),
                Report::Problem(Problem::Refused(refusal)),
                Report::PlanFinished { succeeded: false },
            ]
        );
    }
}

#[tokio::test]
async fn a_pkexec_that_cannot_run_says_why_and_is_no_denial() {
    let launcher = FakeLauncher {
        script: "echo 'pkexec must be setuid root' >&2; exit 127".to_owned(),
        mode: Mode::Run,
    };

    let reports = run(root_plan(), launcher).await;

    assert_eq!(
        reports,
        [
            started(),
            Report::Problem(Problem::HelperFailed {
                kind: HelperFailure::Pkexec,
                detail: "pkexec must be setuid root".to_owned(),
            }),
            Report::PlanFinished { succeeded: false },
        ]
    );
}

#[tokio::test]
async fn user_steps_run_in_the_app_and_skip_in_a_dry_run() {
    let plan = Plan::user(UserAction::UpgradeAur);
    let launcher = FakeLauncher {
        script: "exit 99".to_owned(),
        mode: Mode::DryRun,
    };

    let reports = run(plan, launcher).await;

    assert!(
        reports.contains(&Report::Step(Event::Finished {
            step: StepId::new(1),
            outcome: Outcome::Skipped
        })),
        "{reports:?}"
    );
    assert_eq!(
        reports.last(),
        Some(&Report::PlanFinished { succeeded: true })
    );
}

#[tokio::test]
async fn a_later_segment_does_not_start_after_a_failure() {
    let plan = Plan::new([
        PlannedAction::Root(RootAction::RefreshKeyring),
        PlannedAction::User(UserAction::UpgradeAur),
    ]);

    let launcher = FakeLauncher {
        script: "exit 127".to_owned(),
        mode: Mode::DryRun,
    };

    let reports = run(plan, launcher).await;

    assert!(
        !reports.iter().any(|report| matches!(
            report,
            Report::Step(Event::Started { step }) if *step == StepId::new(2)
        )),
        "{reports:?}"
    );
}

#[tokio::test]
async fn a_helper_that_does_not_start_is_reported_with_the_systems_words() {
    let reports = run(root_plan(), MissingHelper).await;

    assert!(
        reports.iter().any(|report| matches!(
            report,
            Report::Problem(Problem::HelperFailed { kind: HelperFailure::Start, detail })
                if !detail.is_empty()
        )),
        "{reports:?}"
    );
    assert_eq!(
        reports.last(),
        Some(&Report::PlanFinished { succeeded: false })
    );
}
