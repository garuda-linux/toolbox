use std::ffi::OsStr;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use super::{output, run, run_within};
use crate::ProbeError;

const SHORT: Duration = Duration::from_millis(200);
/// Far below the sleeps of the scripts, far above a loaded test machine.
const SOON: Duration = Duration::from_secs(10);

fn shell(script: &str, deadline: Duration) -> Result<super::Finished, ProbeError> {
    run_within(OsStr::new("/bin/sh"), &["-c", script], deadline)
}

fn wait_until_gone(process: &Path, started: Instant) {
    while process.exists() && started.elapsed() < SOON {
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_program_that_ends_in_time_gives_its_output_and_code() {
    let finished = shell("echo out; echo err >&2; exit 3", SOON).unwrap();

    assert_eq!(finished.stdout, "out\n");
    assert_eq!(finished.stderr, "err\n");
    assert_eq!(finished.status.code(), Some(3_i32));
}

#[test]
fn a_program_past_its_deadline_is_stopped_and_has_failed() {
    let started = Instant::now();
    let error = shell("sleep 60", SHORT).err().unwrap();

    assert!(started.elapsed() < SOON, "the program was waited for");
    assert!(
        matches!(&error, ProbeError::Failed(text) if text.contains("no answer") && text.contains("sleep 60")),
        "{error}"
    );
}

#[test]
fn the_children_of_a_stopped_program_are_stopped_too() {
    let folder = tempfile::tempdir().unwrap();
    let noted = folder.path().join("pid");
    let script = format!("sleep 60 & echo $! > {}; wait", noted.display());

    shell(&script, SHORT).err().unwrap();

    let child = std::fs::read_to_string(&noted).unwrap();
    let process = Path::new("/proc").join(child.trim());
    let started = Instant::now();

    // The kill is sent before `run_within` returns, the kernel ends the
    // child a moment later.
    wait_until_gone(&process, started);

    assert!(!process.exists(), "sleep {child} outlived its parent");
}

#[test]
fn output_larger_than_a_pipe_is_kept_whole() {
    let finished = shell(
        "head -c 300000 /dev/zero | tr '\\0' x; head -c 300000 /dev/zero | tr '\\0' y >&2",
        SOON,
    )
    .unwrap();

    assert_eq!(finished.stdout.len(), 300_000);
    assert_eq!(finished.stderr.len(), 300_000);
    assert!(finished.status.success());
}

/// The program ends at once, but what it left running holds its output
/// open. The deadline still ends the wait and stops the leftover.
#[test]
fn a_child_that_keeps_the_output_open_ends_at_the_deadline() {
    let folder = tempfile::tempdir().unwrap();
    let noted = folder.path().join("pid");
    let started = Instant::now();

    let error = shell(
        &format!("sleep 60 & echo $! > {}; echo hi", noted.display()),
        SHORT,
    )
    .err()
    .unwrap();

    assert!(started.elapsed() < SOON, "the output was waited for");
    assert!(
        matches!(&error, ProbeError::Failed(text) if text.contains("no answer")),
        "{error}"
    );

    let pid = std::fs::read_to_string(&noted).unwrap();
    let process = Path::new("/proc").join(pid.trim());

    wait_until_gone(&process, started);

    assert!(!process.exists(), "sleep {pid} outlived the deadline");
}

#[test]
fn a_missing_program_is_unavailable() {
    assert_eq!(
        run(OsStr::new("/nonexistent/program"), &[]).err(),
        Some(ProbeError::Unavailable)
    );
}

#[test]
fn a_failed_program_fails_with_its_first_complaint() {
    let error = output(OsStr::new("/bin/sh"), &["-c", "echo broken >&2; exit 1"]).unwrap_err();

    assert!(matches!(&error, ProbeError::Failed(text) if text.contains("broken")));
}

#[test]
fn a_programs_colour_codes_do_not_reach_the_failure_text() {
    // Progress bars and highlighted warnings are the usual reason a probe
    // fails, and escape sequences in a Label are a hole.
    let error = shell("printf '\\033[1;31mbroken\\033[0m\\n' >&2; exit 1", SOON)
        .unwrap()
        .failure();

    assert!(
        matches!(&error, ProbeError::Failed(text) if text.contains("broken")),
        "{error}"
    );
    assert!(
        matches!(&error, ProbeError::Failed(text) if !text.contains('\u{1b}')),
        "{error}"
    );
}

/// pacman removes its database lock on SIGTERM, a SIGKILL leaves it and
/// every later `checkupdates` fails.
#[test]
fn a_stopped_program_may_clean_up_first() {
    let folder = tempfile::tempdir().unwrap();
    let lock = folder.path().join("db.lck");
    let script = format!(
        "touch {lock}; trap 'rm -f {lock}; exit 1' TERM; sleep 60 & wait",
        lock = lock.display()
    );

    shell(&script, SHORT).err().unwrap();

    assert!(
        !lock.exists(),
        "the program had no chance to remove its lock"
    );
}

/// The grace is five seconds, then the kill reaches the whole group.
#[test]
fn a_program_that_ignores_the_stop_is_killed_after_the_grace() {
    let started = Instant::now();

    shell("trap '' TERM; sleep 60 & wait", SHORT).err().unwrap();

    assert!(
        started.elapsed() < Duration::from_secs(30),
        "the stop waited for the program"
    );
}
