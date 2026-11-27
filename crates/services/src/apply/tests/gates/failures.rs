//! A gated plan when something fails: a root step before the gate, a user
//! step that fails or is cancelled, a missing password, a dismissed dialog and
//! a helper ending at its gate.

use std::time::Duration;

use toolbox_core::features::console::HelperFailure;
use toolbox_plan::protocol::{Answer, Event, Mode, Outcome};
use toolbox_plan::{Plan, PlannedAction, RootAction, UserName};
use toolbox_system::pkexec::Refusal;

use super::super::{finished, run};
use super::fakes::{Act, lines, reaped, serve_when, starts, wire};
use super::{
    failed, finished_as, finished_ok, is_step, is_user_started, marked, refresh, scripted, started,
    three, to_the_gate, user, user_step, waiting,
};
use crate::apply::{ApplyCommand, Problem, Report};

#[tokio::test]
async fn a_root_step_that_fails_before_the_gate_keeps_the_user_step_from_running() {
    let dir = tempfile::tempdir().unwrap();
    let (record, log) = (dir.path().join("stdin"), dir.path().join("log"));
    let launcher = scripted(
        &[
            Act::Hear(record),
            Act::Say(vec![
                Event::Accepted,
                started(1),
                finished_as(1, failed()),
                Event::Done,
            ]),
        ],
        Mode::Run,
        user(&log, Duration::ZERO, Outcome::Succeeded),
    );

    let reports = run(three(), launcher).await;

    assert!(!log.exists(), "the user step ran");
    assert!(!reports.iter().any(is_user_started));
    assert_eq!(reports.last(), Some(&finished(false)));
}

#[tokio::test]
async fn a_failed_user_step_ends_the_helper_at_its_gate() {
    let dir = tempfile::tempdir().unwrap();
    let (record, starts_file) = (dir.path().join("stdin"), dir.path().join("starts"));
    let acts = marked(
        &starts_file,
        vec![
            Act::Hear(record.clone()),
            Act::Say(vec![
                Event::Accepted,
                started(1),
                finished_ok(1),
                waiting(3),
            ]),
            Act::Hear(record.clone()),
            Act::Say(vec![
                started(3),
                finished_as(3, Outcome::Cancelled),
                Event::Done,
            ]),
        ],
    );

    let launcher = scripted(
        &acts,
        Mode::Run,
        user(&dir.path().join("log"), Duration::ZERO, failed()),
    );

    let reports = run(three(), launcher).await;

    assert_eq!(lines(&record).get(1), Some(&wire(&Answer::Cancel)));
    assert!(reaped(&starts_file));
    assert!(reports.contains(&Report::Step(finished_as(3, Outcome::Cancelled))));
    assert_eq!(reports.last(), Some(&finished(false)));
}

#[tokio::test]
async fn a_cancel_during_a_user_step_cancels_it_and_ends_the_helper() {
    let dir = tempfile::tempdir().unwrap();
    let (record, starts_file, log) = (
        dir.path().join("stdin"),
        dir.path().join("starts"),
        dir.path().join("log"),
    );

    let acts = marked(
        &starts_file,
        vec![
            Act::Hear(record.clone()),
            Act::Say(vec![
                Event::Accepted,
                started(1),
                finished_ok(1),
                waiting(3),
            ]),
            Act::Hear(record.clone()),
            Act::Say(vec![
                started(3),
                finished_as(3, Outcome::Cancelled),
                Event::Done,
            ]),
        ],
    );

    let launcher = scripted(
        &acts,
        Mode::Run,
        user(&log, Duration::from_secs(10), Outcome::Succeeded),
    );

    let reports = serve_when(
        launcher,
        three(),
        is_user_started,
        vec![ApplyCommand::Cancel],
    )
    .await;

    assert_eq!(lines(&record).get(1), Some(&wire(&Answer::Cancel)));
    assert!(lines(&log).contains(&"user-end".to_owned()));
    assert!(reaped(&starts_file));
    assert_eq!(reports.last(), Some(&finished(false)));
}

#[tokio::test]
async fn a_missing_password_behind_a_gate_stops_before_the_helper_starts() {
    let dir = tempfile::tempdir().unwrap();
    let (record, starts_file, log) = (
        dir.path().join("stdin"),
        dir.path().join("starts"),
        dir.path().join("log"),
    );

    let plan = Plan::new([
        refresh(),
        user_step(),
        PlannedAction::Root(RootAction::SetPassword {
            user: UserName::new("ada").unwrap(),
            password: None,
        }),
    ]);

    let acts = marked(&starts_file, to_the_gate(&record));
    let launcher = scripted(
        &acts,
        Mode::Run,
        user(&log, Duration::ZERO, Outcome::Succeeded),
    );

    let reports = run(plan, launcher).await;

    assert_eq!(starts(&starts_file), 0);
    assert!(!log.exists(), "the user step ran");
    assert!(reports.contains(&Report::Problem(Problem::HelperFailed {
        kind: HelperFailure::MissingSecret,
        detail: "3".to_owned(),
    })));
    assert_eq!(reports.last(), Some(&finished(false)));
}

#[tokio::test]
async fn a_dismissed_password_dialog_runs_no_step_and_no_user_step() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("log");
    let launcher = scripted(
        &[Act::Raw("exit 126")],
        Mode::Run,
        user(&log, Duration::ZERO, Outcome::Succeeded),
    );

    let reports = run(three(), launcher).await;

    assert!(!log.exists(), "the user step ran");
    assert!(!reports.iter().any(is_step));
    assert!(reports.contains(&Report::Problem(Problem::Refused(Refusal::Dismissed))));
    assert_eq!(reports.last(), Some(&finished(false)));
}

#[tokio::test]
async fn a_helper_that_ends_at_its_gate_is_reported_as_ended() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let launcher = scripted(
        &[
            Act::Hear(record),
            Act::Say(vec![
                Event::Accepted,
                started(1),
                finished_ok(1),
                waiting(3),
            ]),
            Act::Raw("exit 3"),
        ],
        Mode::Run,
        user(&dir.path().join("log"), Duration::ZERO, Outcome::Succeeded),
    );

    let reports = run(three(), launcher).await;

    assert!(reports.iter().any(|report| matches!(
        report,
        Report::Problem(Problem::HelperFailed {
            kind: HelperFailure::Ended,
            detail: _,
        })
    )));
    assert_eq!(reports.last(), Some(&finished(false)));
}
