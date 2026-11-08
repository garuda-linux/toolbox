use super::{PageRun, RunKind};
use crate::testing::user_name;

fn running(kind: RunKind, user: &str) -> PageRun {
    PageRun::Running(kind, user_name(user))
}

#[test]
fn a_run_ends_in_the_outcome_of_its_plan() {
    let run = || running(RunKind::Password, "ada");

    assert_eq!(PageRun::default().state(), "");
    assert_eq!(run().state(), "running");
    assert_eq!(run().user(), "ada");
    assert_eq!(run().kind(), "password");
    assert_eq!(run().finished(true).state(), "done");
    assert_eq!(run().finished(false).state(), "failed");
    assert_eq!(run().finished(false).user(), "ada");
    assert_eq!(run().finished(false).kind(), "password");
}

#[test]
fn a_new_account_is_a_run_of_its_own_kind() {
    let run = running(RunKind::NewAccount, "grace");

    assert_eq!(run.kind(), "new-account");
    assert_eq!(run.finished(true).kind(), "new-account");
    assert_eq!(PageRun::default().kind(), "");
}

/// Plans run one after the other, so a sent run that another plan's end
/// meets has not started and takes no outcome from it.
#[test]
fn a_sent_run_takes_no_outcome_before_it_started() {
    let sent = || PageRun::Sent(RunKind::Password, user_name("ada"));

    assert_eq!(sent().state(), "running");
    assert_eq!(sent().finished(true).state(), "running");
    assert_eq!(sent().started().finished(true).state(), "done");
}

#[test]
fn a_refused_run_shows_nothing() {
    let sent = PageRun::Sent(RunKind::Password, user_name("ada"));

    assert_eq!(sent.refused(), PageRun::None);
    assert_eq!(
        running(RunKind::Password, "ada").refused().state(),
        "running"
    );
}

/// Only the run the page started has an outcome to show. A later plan,
/// such as an apply of the queue, leaves the shown outcome alone.
#[test]
fn only_a_running_run_takes_an_outcome() {
    assert_eq!(PageRun::default().finished(true).state(), "");
    assert_eq!(
        running(RunKind::Password, "ada")
            .finished(true)
            .finished(false)
            .state(),
        "done"
    );
}
