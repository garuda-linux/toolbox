use std::cell::Cell;
use std::time::{Duration, Instant};

use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::{Event, Failure, Outcome};
use toolbox_plan::{PackageName, Plan, PlannedAction, RootAction, StepId, TargetList, UserAction};
use toolbox_services::apply::{Problem, Report};
use toolbox_system::pkexec::Refusal;

use super::{Clock, Events, error, needs_typing, planned};

fn ticking(tick: Duration) -> Clock {
    let start = Instant::now();
    let reads = Cell::new(0_u32);

    Box::new(move || {
        let read = reads.get();
        reads.set(read + 1);
        start + tick * read
    })
}

fn lines(reports: &[Report]) -> String {
    let mut events = Events::new(ticking(Duration::from_millis(4_200)));
    reports.iter().map(|report| events.report(report)).collect()
}

fn two_steps() -> Report {
    Report::PlanStarted {
        steps: vec![
            (StepId::new(1), "pacman-key --refresh-keys".to_owned()),
            (
                StepId::new(2),
                "systemctl restart -- sshd.service".to_owned(),
            ),
        ],
    }
}

fn step(id: u32, event: impl FnOnce(StepId) -> Event) -> Report {
    Report::Step(event(StepId::new(id)))
}

#[test]
fn the_plan_is_one_line_with_every_step() {
    let plan = Plan::new([
        PlannedAction::Root(RootAction::RefreshKeyring),
        PlannedAction::User(UserAction::GarudaHealth),
    ]);

    let line = planned(&plan);

    assert!(
        line.starts_with(r#"{"schema":1,"event":"plan","steps":[{"step":1,"command":"pacman-key"#),
        "{line}"
    );
    assert!(line.contains(r#"},{"step":2,"command":""#), "{line}");
    assert!(line.ends_with("}]}\n"), "{line}");
    assert_eq!(line.lines().count(), 1, "{line}");
}

#[test]
fn an_empty_plan_has_no_steps() {
    assert_eq!(
        planned(&Plan::new([])),
        "{\"schema\":1,\"event\":\"plan\",\"steps\":[]}\n"
    );
}

#[test]
fn a_run_is_started_output_finished_and_done() {
    let text = lines(&[
        two_steps(),
        step(2, |step| Event::Started { step }),
        step(2, |step| Event::Output {
            step,
            data: "one\ntw".to_owned(),
        }),
        step(2, |step| Event::Output {
            step,
            data: "o\r\nthree".to_owned(),
        }),
        step(2, |step| Event::Finished {
            step,
            outcome: Outcome::Succeeded,
        }),
        Report::PlanFinished { succeeded: true },
    ]);

    assert_eq!(
        text,
        concat!(
            r#"{"schema":1,"event":"started","step":2,"of":2,"command":"systemctl restart -- sshd.service"}"#,
            "\n",
            r#"{"schema":1,"event":"output","step":2,"line":"one"}"#,
            "\n",
            r#"{"schema":1,"event":"output","step":2,"line":"two"}"#,
            "\n",
            r#"{"schema":1,"event":"output","step":2,"line":"three"}"#,
            "\n",
            r#"{"schema":1,"event":"finished","step":2,"outcome":"done","seconds":4.2}"#,
            "\n",
            r#"{"schema":1,"event":"done","ok":true}"#,
            "\n",
        )
    );
}

#[test]
fn every_outcome_has_its_word() {
    for (outcome, word) in [
        (
            Outcome::Failed {
                cause: Failure::ExitCode { code: 1_i32 },
            },
            "failed",
        ),
        (Outcome::Cancelled, "cancelled"),
        (Outcome::Skipped, "skipped"),
    ] {
        let text = lines(&[
            two_steps(),
            step(1, |step| Event::Started { step }),
            step(1, |step| Event::Finished { step, outcome }),
        ]);

        let wanted = format!(
            r#"{{"schema":1,"event":"finished","step":1,"outcome":"{word}","seconds":4.2}}"#
        );

        assert!(text.contains(&wanted), "{text}");
    }
}

#[test]
fn a_step_that_never_started_took_no_time() {
    let text = lines(&[
        two_steps(),
        step(1, |step| Event::Finished {
            step,
            outcome: Outcome::Skipped,
        }),
    ]);

    assert_eq!(
        text,
        "{\"schema\":1,\"event\":\"finished\",\"step\":1,\"outcome\":\"skipped\",\"seconds\":0.0}\n"
    );
}

#[test]
fn a_helper_failure_carries_the_program_text_without_verbose() {
    let text = lines(&[
        Report::Problem(Problem::HelperFailed {
            kind: HelperFailure::Start,
            detail: "pkexec: not found\n".to_owned(),
        }),
        Report::PlanFinished { succeeded: false },
    ]);

    assert_eq!(
        text,
        concat!(
            r#"{"schema":1,"event":"error","message":"the helper did not start","detail":"pkexec: not found\n"}"#,
            "\n",
            r#"{"schema":1,"event":"done","ok":false}"#,
            "\n",
        )
    );
}

#[test]
fn a_refusal_names_what_helps() {
    let text = lines(&[Report::Problem(Problem::Refused(Refusal::Dismissed))]);

    assert_eq!(
        text,
        concat!(
            r#"{"schema":1,"event":"error","message":"not carried out: the password dialog was closed","#,
            r#""hint":"run it again and enter your password"}"#,
            "\n",
        )
    );
}

#[test]
fn a_rejected_request_and_a_busy_service_are_errors() {
    let text = lines(&[
        step(0, |_| Event::Rejected {
            reason: "unknown action".to_owned(),
        }),
        Report::Busy,
    ]);

    assert_eq!(
        text,
        concat!(
            r#"{"schema":1,"event":"error","message":"the helper refused the change","detail":"unknown action"}"#,
            "\n",
            r#"{"schema":1,"event":"error","message":"another change is still running"}"#,
            "\n",
        )
    );
}

#[test]
fn an_error_line_splits_off_its_hint() {
    assert_eq!(
        error("no unit sshd\ndid you mean `sshd.service`?", None),
        concat!(
            r#"{"schema":1,"event":"error","message":"no unit sshd","hint":"did you mean `sshd.service`?"}"#,
            "\n"
        )
    );
}

#[test]
fn paru_needs_typing_and_a_root_step_does_not() {
    assert!(needs_typing(&PlannedAction::User(UserAction::UpgradeAur)));
    assert!(needs_typing(&PlannedAction::User(UserAction::InstallAur {
        packages: TargetList::one(PackageName::new("yay-bin").unwrap()),
    })));
    assert!(!needs_typing(&PlannedAction::User(
        UserAction::GarudaHealth
    )));
    assert!(!needs_typing(&PlannedAction::Root(
        RootAction::RefreshKeyring
    )));
}
