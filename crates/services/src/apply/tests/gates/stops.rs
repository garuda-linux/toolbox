//! A gated plan when the user asks it to stop and the running step still
//! ends well: nothing behind it may start.

use std::time::Duration;

use toolbox_plan::protocol::{Answer, Event, Mode, Outcome};

use super::super::finished;
use super::fakes::{Act, lines, serve_when, wire};
use super::{
    finished_as, finished_ok, is_user_started, marked, scripted, started, three, user, waiting,
};
use crate::apply::{ApplyCommand, Report};

/// The command line's first Ctrl+C during a build says "stopping after this
/// step". The build ends well, and the root step behind the gate, such as
/// the install of what was built, must not run.
#[tokio::test]
async fn a_finish_during_a_user_step_keeps_the_gated_root_step_from_running() {
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
        user(&log, Duration::from_millis(400), Outcome::Succeeded),
    );

    let reports = serve_when(
        launcher,
        three(),
        is_user_started,
        vec![ApplyCommand::Finish],
    )
    .await;

    assert_eq!(lines(&record).get(1), Some(&wire(&Answer::Cancel)));
    assert!(reports.contains(&Report::Step(finished_as(3, Outcome::Cancelled))));
    assert_eq!(reports.last(), Some(&finished(false)));
}

/// A finish during the last root step of a segment lets that step end. The
/// user step behind it, such as a build of an hour, must not start.
#[tokio::test]
async fn a_finish_during_a_root_step_keeps_the_next_user_step_from_starting() {
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
            Act::Say(vec![Event::Accepted, started(1)]),
            Act::Hear(record.clone()),
            Act::Say(vec![
                finished_ok(1),
                started(3),
                finished_as(3, Outcome::Cancelled),
                Event::Done,
            ]),
        ],
    );

    let launcher = scripted(
        &acts,
        Mode::Run,
        user(&log, Duration::ZERO, Outcome::Succeeded),
    );

    let reports = serve_when(
        launcher,
        three(),
        |report| *report == Report::Step(started(1)),
        vec![ApplyCommand::Finish],
    )
    .await;

    assert_eq!(lines(&record).get(1), Some(&wire(&Answer::Finish)));
    assert!(!log.exists(), "the user step ran");
    assert!(!reports.iter().any(is_user_started));
    assert_eq!(reports.last(), Some(&finished(false)));
}
