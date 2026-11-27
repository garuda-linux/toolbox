use super::{PageRun, Retry, RetryStep, RunKind};
use crate::test_support::user_name;

fn running(kind: RunKind, user: &str) -> PageRun {
    PageRun::Running(kind, user_name(user))
}

fn retry() -> Retry<&'static str> {
    Retry::new("form", RunKind::NewAccount, user_name("hopper"))
}

fn step(retry: Retry<&'static str>, run: &PageRun) -> Retry<&'static str> {
    match retry.step(run) {
        RetryStep::Wait(waiting) => waiting,
        RetryStep::Reopen(form) => panic!("reopened {form} at {run:?}"),
        RetryStep::Forget => panic!("dropped at {run:?}"),
    }
}

/// The add-user dialog closes when it sends the account and opens again,
/// without its passwords, when the account could not be added.
#[test]
fn a_failed_run_gives_its_form_back() {
    let sent = step(
        retry(),
        &PageRun::Sent(RunKind::NewAccount, user_name("hopper")),
    );

    let started = step(sent, &running(RunKind::NewAccount, "hopper"));

    assert_eq!(
        started.step(&PageRun::Failed(RunKind::NewAccount, user_name("hopper"))),
        RetryStep::Reopen("form")
    );
}

#[test]
fn a_done_run_drops_its_form() {
    let started = step(retry(), &running(RunKind::NewAccount, "hopper"));

    assert_eq!(
        started.step(&PageRun::Done(RunKind::NewAccount, user_name("hopper"))),
        RetryStep::Forget
    );
}

/// The helper refused the run because another plan ran: nothing was added.
#[test]
fn a_refused_run_gives_its_form_back() {
    let sent = step(
        retry(),
        &PageRun::Sent(RunKind::NewAccount, user_name("hopper")),
    );

    assert_eq!(sent.step(&PageRun::None), RetryStep::Reopen("form"));
}

/// Until its own run started, the outcome of an earlier try or of another
/// run is not this form's.
#[test]
fn an_earlier_outcome_is_not_the_forms() {
    let before = step(
        retry(),
        &PageRun::Failed(RunKind::NewAccount, user_name("hopper")),
    );

    let cleared = step(before, &PageRun::None);

    let other = step(cleared, &running(RunKind::Password, "grace"));

    assert_eq!(
        other.step(&PageRun::Failed(RunKind::Password, user_name("grace"))),
        RetryStep::Wait(retry())
    );
}

#[test]
fn a_run_ends_in_the_outcome_of_its_plan() {
    let run = || running(RunKind::Password, "grace");

    assert_eq!(PageRun::default().state(), "");
    assert_eq!(run().state(), "running");
    assert_eq!(run().user(), "grace");
    assert_eq!(run().kind(), "password");
    assert_eq!(run().finished(true).state(), "done");
    assert_eq!(run().finished(false).state(), "failed");
    assert_eq!(run().finished(false).user(), "grace");
    assert_eq!(run().finished(false).kind(), "password");
}

#[test]
fn a_new_account_is_a_run_of_its_own_kind() {
    let run = running(RunKind::NewAccount, "hopper");

    assert_eq!(run.kind(), "new-account");
    assert_eq!(run.finished(true).kind(), "new-account");
    assert_eq!(PageRun::default().kind(), "");
}

/// Plans run one after the other, so a sent run that another plan's end
/// meets has not started and takes no outcome from it.
#[test]
fn a_sent_run_takes_no_outcome_before_it_started() {
    let sent = || PageRun::Sent(RunKind::Password, user_name("grace"));

    assert_eq!(sent().state(), "running");
    assert_eq!(sent().finished(true).state(), "running");
    assert_eq!(sent().started().finished(true).state(), "done");
}

#[test]
fn a_refused_run_shows_nothing() {
    let sent = PageRun::Sent(RunKind::Password, user_name("grace"));

    assert_eq!(sent.refused(), PageRun::None);
    assert_eq!(
        running(RunKind::Password, "grace").refused().state(),
        "running"
    );
}

/// Only the run the page started has an outcome to show. A later plan,
/// such as an apply of the queue, leaves the shown outcome alone.
#[test]
fn only_a_running_run_takes_an_outcome() {
    assert_eq!(PageRun::default().finished(true).state(), "");
    assert_eq!(
        running(RunKind::Password, "grace")
            .finished(true)
            .finished(false)
            .state(),
        "done"
    );
}
