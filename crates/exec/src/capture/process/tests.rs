use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::capture::{
    CaptureError, Exited, MAX_ERROR_BYTES, MAX_INPUT_BYTES, Policy, run_captured,
};
use crate::pty_runner::Environment;
use crate::test_support::gone;

const SHORT: Duration = Duration::from_millis(200);
/// Far below the sleeps of the scripts, far above a loaded test machine.
const SOON: Duration = Duration::from_secs(10);

fn shell(script: &str, policy: &Policy<'_>) -> Result<Exited, CaptureError> {
    let mut command = Command::new("/bin/sh");

    command.args(["-c", script]);
    run_captured(
        &mut command,
        Environment::Inherited,
        &[],
        policy,
        &Arc::default(),
    )
}

/// A script that holds `lock` and removes it on SIGTERM, as pacman does.
fn locking(lock: &Path) -> String {
    format!(
        "touch {lock}; trap 'rm -f {lock}; exit 1' TERM; sleep 60 & wait",
        lock = lock.display()
    )
}

#[test]
fn past_the_deadline_the_program_and_its_group_are_killed() {
    let folder = tempfile::tempdir().unwrap();
    let noted = folder.path().join("pid");
    let started = Instant::now();
    let result = shell(
        &format!("sleep 60 & echo $! > {}; wait", noted.display()),
        &Policy::whole().within(SHORT),
    );

    assert!(matches!(result, Err(CaptureError::TimedOut)), "{result:?}");
    assert!(started.elapsed() < SOON, "the program was waited for");
    assert!(gone(&noted), "the child outlived the deadline");
}

#[test]
fn with_a_grace_the_program_may_clean_up_first() {
    let folder = tempfile::tempdir().unwrap();
    let lock = folder.path().join("db.lck");
    let result = shell(
        &locking(&lock),
        &Policy::whole().within(SHORT).term_first(SOON),
    );

    assert!(matches!(result, Err(CaptureError::TimedOut)), "{result:?}");
    assert!(!lock.exists(), "the program had no chance to clean up");
}

#[test]
fn without_a_grace_the_kill_comes_at_once() {
    let folder = tempfile::tempdir().unwrap();
    let lock = folder.path().join("db.lck");

    shell(&locking(&lock), &Policy::whole().within(SHORT)).unwrap_err();

    assert!(lock.exists(), "the program got a SIGTERM");
}

#[test]
fn a_program_that_ignores_the_request_is_killed_after_the_grace() {
    let folder = tempfile::tempdir().unwrap();
    let noted = folder.path().join("pid");
    let started = Instant::now();

    // An ignored signal stays ignored in the children too.
    let script = format!(
        "trap '' TERM; sleep 60 & echo $! > {}; wait",
        noted.display()
    );

    let result = shell(&script, &Policy::whole().within(SHORT).term_first(SHORT));

    assert!(matches!(result, Err(CaptureError::TimedOut)), "{result:?}");
    assert!(started.elapsed() < SOON, "the grace was not kept short");
    assert!(gone(&noted), "the child that ignored SIGTERM still runs");
}

#[test]
fn a_capped_output_past_its_limit_is_an_error() {
    let result = shell("seq 1 100000", &Policy::tail(1000).capped());

    assert!(matches!(result, Err(CaptureError::TooLarge)), "{result:?}");
}

#[test]
fn an_uncapped_output_past_its_limit_keeps_its_end() {
    let exited = shell("seq 1 100000", &Policy::tail(1000)).unwrap();

    assert!(exited.truncated);
    assert!(exited.output.len() <= 1000);
    assert!(exited.output.ends_with(b"100000\n"));
}

#[test]
fn a_capped_output_within_its_limit_is_whole() {
    let exited = shell("seq 1 3", &Policy::tail(1000).capped()).unwrap();

    assert!(!exited.truncated);
    assert_eq!(exited.output, b"1\n2\n3\n");
}

/// A program that never stops printing would otherwise run until the
/// deadline, or forever without one.
#[test]
fn a_capped_program_that_keeps_printing_is_stopped_at_the_limit() {
    let started = Instant::now();
    let result = shell(
        "trap '' PIPE; while :; do echo line; done",
        &Policy::tail(1000).capped(),
    );

    assert!(matches!(result, Err(CaptureError::TooLarge)), "{result:?}");
    assert!(started.elapsed() < SOON);
}

#[test]
fn the_whole_policy_keeps_long_complaints() {
    let exited = shell(
        "head -c 300000 /dev/zero | tr '\\0' y >&2",
        &Policy::whole(),
    )
    .unwrap();

    assert_eq!(exited.errors.len(), 300_000);
}

#[test]
fn the_tail_policy_cuts_long_complaints() {
    let exited = shell("seq 1 200000 >&2", &Policy::tail(64)).unwrap();

    assert!(exited.errors.len() <= MAX_ERROR_BYTES);
    assert!(exited.errors.ends_with(b"200000\n"));
}

#[test]
fn fed_input_reaches_the_program_and_then_its_end() {
    let exited = shell(
        "read -r line; echo \"got $line\"; cat",
        &Policy::whole().within(SOON).fed(b"secret\nrest"),
    )
    .unwrap();

    assert!(exited.status.success());
    assert_eq!(exited.output, b"got secret\nrest");
}

#[test]
fn a_program_that_never_reads_its_input_still_reports_its_exit() {
    let exited = shell("exit 7", &Policy::whole().fed(b"unread\n")).unwrap();

    assert_eq!(exited.status.code(), Some(7_i32));
}

#[test]
fn input_longer_than_a_pipe_holds_is_refused_before_the_start() {
    let folder = tempfile::tempdir().unwrap();
    let marker = folder.path().join("ran");
    let input = vec![b'x'; MAX_INPUT_BYTES + 1];
    let result = shell(
        &format!("touch {}", marker.display()),
        &Policy::whole().fed(&input),
    );

    assert!(matches!(result, Err(CaptureError::Failed(_))), "{result:?}");
    assert!(!marker.exists());
}

#[test]
fn a_deadline_too_far_away_never_passes() {
    let exited = shell("echo ok", &Policy::whole().within(Duration::MAX)).unwrap();

    assert_eq!(exited.output, b"ok\n");
}
