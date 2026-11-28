//! A plan with several root segments: one helper session, gated at the
//! first step of each later root segment.

mod failures;
mod fakes;
mod stops;

use std::path::Path;
use std::time::Duration;

use toolbox_plan::protocol::{Answer, Event, Failure, Mode, Outcome, Request, TerminalSize};
use toolbox_plan::{Password, Plan, PlannedAction, RootAction, StepId, UserAction, UserName};

use self::fakes::{Act, Scripted, UserFake, helper, lines, serve_when, starts, wire};
use super::{finished, run};
use crate::apply::{ApplyCommand, Report};

const SECRET: &str = "correct horse battery staple";

pub(super) fn step(number: u32) -> StepId {
    StepId::new(number)
}

pub(super) fn refresh() -> PlannedAction {
    PlannedAction::Root(RootAction::RefreshKeyring)
}

pub(super) fn user_step() -> PlannedAction {
    PlannedAction::User(UserAction::UpgradeAur)
}

/// Root, user, root: the second root step is gated.
pub(super) fn three() -> Plan {
    Plan::new([refresh(), user_step(), refresh()])
}

pub(super) fn started(number: u32) -> Event {
    Event::Started { step: step(number) }
}

pub(super) fn finished_ok(number: u32) -> Event {
    Event::Finished {
        step: step(number),
        outcome: Outcome::Succeeded,
    }
}

pub(super) fn finished_as(number: u32, outcome: Outcome) -> Event {
    Event::Finished {
        step: step(number),
        outcome,
    }
}

pub(super) fn failed() -> Outcome {
    Outcome::Failed {
        cause: Failure::ExitCode { code: 1_i32 },
        cleanup_failed: Vec::new(),
    }
}

pub(super) fn waiting(number: u32) -> Event {
    Event::Waiting { step: step(number) }
}

pub(super) fn proceed(number: u32) -> String {
    wire(&Answer::Proceed { step: step(number) })
}

/// The helper of [`three`] when both root steps succeed: it reads the
/// request, waits at the gate and reads the answer that passes it.
pub(super) fn to_the_gate(record: &Path) -> Vec<Act> {
    vec![
        Act::Hear(record.to_path_buf()),
        Act::Say(vec![
            Event::Accepted,
            started(1),
            finished_ok(1),
            waiting(3),
        ]),
        Act::Hear(record.to_path_buf()),
        Act::Say(vec![started(3), finished_ok(3), Event::Done]),
    ]
}

/// Puts a start mark in front of a helper script.
pub(super) fn marked(starts_file: &Path, mut acts: Vec<Act>) -> Vec<Act> {
    acts.insert(0, Act::Mark(starts_file.to_path_buf()));
    acts
}

pub(super) fn user(log: &Path, hold: Duration, outcome: Outcome) -> UserFake {
    UserFake {
        log: log.to_path_buf(),
        hold,
        outcome,
    }
}

pub(super) fn scripted(acts: &[Act], mode: Mode, user: UserFake) -> Scripted {
    Scripted {
        script: helper(acts),
        mode,
        user,
    }
}

pub(super) fn is_step(report: &Report) -> bool {
    matches!(report, Report::Step(_))
}

pub(super) fn is_user_started(report: &Report) -> bool {
    matches!(report, Report::Step(Event::Started { step: id }) if *id == step(2))
}

pub(super) fn is_root_started(report: &Report) -> bool {
    matches!(report, Report::Step(Event::Started { step: id }) if *id == step(1))
}

#[tokio::test]
async fn a_plan_with_two_root_segments_starts_one_helper_with_both_steps_and_a_gate() {
    let dir = tempfile::tempdir().unwrap();
    let (record, starts_file) = (dir.path().join("stdin"), dir.path().join("starts"));
    let acts = marked(&starts_file, to_the_gate(&record));
    let launcher = scripted(
        &acts,
        Mode::Run,
        user(&dir.path().join("log"), Duration::ZERO, Outcome::Succeeded),
    );

    let reports = run(three(), launcher).await;

    assert_eq!(starts(&starts_file), 1);
    let received = lines(&record);
    let request = Request::parse(received.first().unwrap()).unwrap();
    assert_eq!(
        request.steps.iter().map(|root| root.id).collect::<Vec<_>>(),
        [step(1), step(3)]
    );
    assert_eq!(request.gates, [step(3)]);
    assert_eq!(received.get(1), Some(&proceed(3)));
    assert!(!reports.contains(&Report::Step(waiting(3))));
    assert_eq!(reports.last(), Some(&finished(true)));
}

#[tokio::test]
async fn the_gated_step_waits_for_a_user_step_the_helper_waited_before() {
    let dir = tempfile::tempdir().unwrap();
    let (record, log) = (dir.path().join("stdin"), dir.path().join("log"));
    let mut acts = to_the_gate(&record);
    acts.insert(
        3,
        Act::Note {
            log: log.clone(),
            text: "proceed",
        },
    );
    let launcher = scripted(
        &acts,
        Mode::Run,
        user(&log, Duration::from_millis(300), Outcome::Succeeded),
    );

    let reports = run(three(), launcher).await;

    let notes = lines(&log);
    let user_ended = notes.iter().position(|line| line == "user-end");
    let proceeded = notes.iter().position(|line| line == "proceed");
    assert!(user_ended.is_some() && user_ended < proceeded, "{notes:?}");
    assert_eq!(reports.last(), Some(&finished(true)));
}

#[tokio::test]
async fn the_app_waits_for_the_gate_when_the_user_step_ends_first() {
    let dir = tempfile::tempdir().unwrap();
    let (record, log) = (dir.path().join("stdin"), dir.path().join("log"));
    let launcher = scripted(
        &[
            Act::Hear(record.clone()),
            Act::Say(vec![Event::Accepted, started(1), finished_ok(1)]),
            Act::Pause,
            Act::Seen {
                record: record.clone(),
                log: log.clone(),
            },
            Act::Say(vec![waiting(3)]),
            Act::Hear(record),
            Act::Note {
                log: log.clone(),
                text: "proceed",
            },
            Act::Say(vec![started(3), finished_ok(3), Event::Done]),
        ],
        Mode::Run,
        user(&log, Duration::ZERO, Outcome::Succeeded),
    );

    let reports = run(three(), launcher).await;

    assert_eq!(lines(&log), ["user-start", "user-end", "seen 1", "proceed"]);
    assert_eq!(reports.last(), Some(&finished(true)));
}

#[tokio::test]
async fn keys_and_resizes_during_a_user_step_stay_with_the_user_step() {
    let dir = tempfile::tempdir().unwrap();
    let (record, log) = (dir.path().join("stdin"), dir.path().join("log"));
    let launcher = scripted(
        &to_the_gate(&record),
        Mode::Run,
        user(&log, Duration::from_millis(300), Outcome::Succeeded),
    );

    let reports = serve_when(
        launcher,
        three(),
        is_user_started,
        vec![
            ApplyCommand::Input(b"k".to_vec()),
            ApplyCommand::Resize(TerminalSize::from((80, 24))),
        ],
    )
    .await;

    let received = lines(&record);
    assert_eq!(received.len(), 2, "{received:?}");
    assert_eq!(received.get(1), Some(&proceed(3)));
    assert!(lines(&log).contains(&"input k".to_owned()));
    assert!(lines(&log).contains(&"resize 80 24".to_owned()));
    assert_eq!(reports.last(), Some(&finished(true)));
}

#[tokio::test]
async fn a_root_step_still_gets_the_keys_typed_during_it() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let launcher = scripted(
        &[
            Act::Hear(record.clone()),
            Act::Say(vec![Event::Accepted, started(1)]),
            Act::Hear(record.clone()),
            Act::Say(vec![finished_ok(1), waiting(3)]),
            Act::Hear(record.clone()),
            Act::Say(vec![started(3), finished_ok(3), Event::Done]),
        ],
        Mode::Run,
        user(&dir.path().join("log"), Duration::ZERO, Outcome::Succeeded),
    );

    let reports = serve_when(
        launcher,
        three(),
        is_root_started,
        vec![ApplyCommand::Input(b"k".to_vec())],
    )
    .await;

    let received = lines(&record);
    let typed = Answer::Input {
        data: "k".to_owned(),
    };

    assert_eq!(received.get(1), Some(&wire(&typed)));
    assert_eq!(received.get(2), Some(&proceed(3)));
    assert_eq!(reports.last(), Some(&finished(true)));
}

#[tokio::test]
async fn a_dry_run_waits_at_its_gate_too() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let launcher = scripted(
        &[
            Act::Hear(record.clone()),
            Act::Say(vec![
                Event::Accepted,
                started(1),
                finished_as(1, Outcome::Skipped),
                waiting(3),
            ]),
            Act::Hear(record.clone()),
            Act::Say(vec![
                started(3),
                finished_as(3, Outcome::Skipped),
                Event::Done,
            ]),
        ],
        Mode::DryRun,
        user(&dir.path().join("log"), Duration::ZERO, Outcome::Succeeded),
    );

    let reports = run(three(), launcher).await;

    assert_eq!(lines(&record).get(1), Some(&proceed(3)));
    assert_eq!(reports.last(), Some(&finished(true)));
}

#[tokio::test]
async fn a_password_for_a_gated_step_arrives_with_the_request() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("stdin");
    let secret = Password::new(SECRET.to_owned()).unwrap();
    let plan = Plan::new([
        refresh(),
        user_step(),
        PlannedAction::Root(RootAction::SetPassword {
            user: UserName::new("ada").unwrap(),
            password: Some(secret),
        }),
    ]);

    let mut acts = to_the_gate(&record);
    acts.insert(1, Act::Hear(record.clone()));
    let launcher = scripted(
        &acts,
        Mode::Run,
        user(&dir.path().join("log"), Duration::ZERO, Outcome::Succeeded),
    );

    let reports = run(plan, launcher).await;

    let received = lines(&record);
    assert_eq!(received.get(1), Some(&format!("secret 3 {SECRET}")));
    assert_eq!(received.get(2), Some(&proceed(3)));
    assert!(!format!("{reports:?}").contains(SECRET));
    assert_eq!(reports.last(), Some(&finished(true)));
}

#[tokio::test]
async fn a_plan_that_starts_with_a_user_step_starts_the_helper_after_it() {
    let dir = tempfile::tempdir().unwrap();
    let (record, starts_file) = (dir.path().join("stdin"), dir.path().join("starts"));
    let plan = Plan::new([user_step(), refresh()]);
    let acts = marked(
        &starts_file,
        vec![
            Act::Hear(record.clone()),
            Act::Say(vec![
                Event::Accepted,
                started(2),
                finished_ok(2),
                Event::Done,
            ]),
        ],
    );

    let launcher = scripted(
        &acts,
        Mode::Run,
        user(&dir.path().join("log"), Duration::ZERO, Outcome::Succeeded),
    );

    let reports = run(plan, launcher).await;

    assert_eq!(starts(&starts_file), 1);
    let request = Request::parse(lines(&record).first().unwrap()).unwrap();
    assert_eq!(request.gates, Vec::<StepId>::new());
    assert_eq!(reports.last(), Some(&finished(true)));
}

#[tokio::test]
async fn a_declined_first_user_step_never_starts_the_helper() {
    let dir = tempfile::tempdir().unwrap();
    let starts_file = dir.path().join("starts");
    let plan = Plan::new([user_step(), refresh()]);
    let launcher = scripted(
        &[Act::Mark(starts_file.clone())],
        Mode::Run,
        user(&dir.path().join("log"), Duration::ZERO, Outcome::Cancelled),
    );

    let reports = run(plan, launcher).await;

    assert_eq!(starts(&starts_file), 0);
    assert!(
        !reports
            .iter()
            .any(|report| matches!(report, Report::Problem(_)))
    );
    assert_eq!(reports.last(), Some(&finished(false)));
}
