mod broken;
mod identified;

use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::{CaptureError, Captured, MAX_ERROR_BYTES, Policy, Tail, capture};
use crate::pty_runner::Environment;
use crate::test_support::gone;

const LIMIT: usize = 1 << 20;

fn shell(script: &str) -> Command {
    let mut command = Command::new("/bin/sh");

    command.args(["-c", script]);
    command
}

fn run(script: &str, limit: usize) -> Captured {
    capture(
        &mut shell(script),
        Environment::Inherited,
        &[],
        &Policy::tail(limit),
        &Arc::default(),
    )
    .unwrap()
}

#[test]
fn output_errors_and_the_exit_code_are_kept_apart() {
    let captured = run("echo out; echo err >&2; exit 3", LIMIT);

    assert_eq!(
        captured,
        Captured {
            output: "out\n".to_owned(),
            errors: "err\n".to_owned(),
            code: Some(3_i32),
            truncated: false,
        }
    );
}

#[test]
fn of_a_long_output_the_last_whole_lines_stay() {
    let captured = run("seq 1 100000", 1000);
    let lines: Vec<&str> = captured.output.lines().collect();

    assert!(captured.truncated);
    assert!(captured.output.len() <= 1000);
    assert_eq!(lines.last(), Some(&"100000"));
    assert!(lines.len() > 100);

    for (line, next) in lines.iter().zip(lines.iter().skip(1)) {
        assert_eq!(
            line.parse::<u32>().unwrap() + 1,
            next.parse::<u32>().unwrap()
        );
    }
}

#[test]
fn a_short_output_is_not_cut_at_its_first_line() {
    let captured = run("printf 'a\\nb\\n'", 1000);

    assert_eq!(captured.output, "a\nb\n");
    assert!(!captured.truncated);
}

#[test]
fn a_cut_inside_a_character_or_a_single_long_line_stays_text() {
    let mut tail = Tail::new(5);

    tail.push("aé€z".as_bytes());

    let (bytes, truncated) = tail.into_bytes();
    let text = String::from_utf8_lossy(&bytes);

    assert!(truncated);
    assert!(text.ends_with('z'));
    assert!(text.chars().count() <= 3, "{text}");
}

#[test]
fn a_program_that_fills_both_pipes_does_not_block() {
    let captured = run("seq 1 200000 >&2; seq 1 200000", 64);

    assert_eq!(captured.code, Some(0_i32));
    assert!(captured.output.ends_with("200000\n"));
    assert!(captured.errors.ends_with("200000\n"));
    assert!(captured.errors.len() <= MAX_ERROR_BYTES);
}

#[test]
fn binary_output_becomes_lossy_text() {
    let captured = run("printf 'a\\377b\\n'", LIMIT);

    assert_eq!(captured.output, "a\u{fffd}b\n");
}

#[test]
fn a_missing_program_is_told_apart_from_a_failing_one() {
    let missing = capture(
        &mut Command::new("/nonexistent/toolbox-test-program"),
        Environment::Inherited,
        &[],
        &Policy::tail(LIMIT),
        &Arc::default(),
    );

    assert!(matches!(missing, Err(CaptureError::Missing)), "{missing:?}");
    assert_eq!(run("exit 1", LIMIT).code, Some(1_i32));
}

#[test]
fn a_signal_leaves_no_exit_code() {
    assert_eq!(run("kill -TERM $$", LIMIT).code, None);
}

#[test]
fn the_clean_environment_drops_the_callers_variables() {
    let mut command = shell("echo \"$TOOLBOX_TEST_LEAK|$LANG|$PATH\"");

    command.env("TOOLBOX_TEST_LEAK", "leaked");

    let captured = capture(
        &mut command,
        Environment::Clean,
        &[],
        &Policy::tail(LIMIT),
        &Arc::default(),
    )
    .unwrap();

    assert_eq!(
        captured.output,
        "|C.UTF-8|/usr/local/sbin:/usr/local/bin:/usr/bin\n"
    );
}

#[test]
fn the_program_gets_no_input() {
    assert_eq!(run("cat; echo done", LIMIT).output, "done\n");
}

#[test]
fn a_cancel_ends_the_wait_and_kills_the_programs_group() {
    let cancel = Arc::new(AtomicBool::new(false));
    let setter = Arc::clone(&cancel);
    let started = Instant::now();

    std::thread::Builder::new()
        .name("test-cancel".to_owned())
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            setter.store(true, Ordering::Relaxed);
        })
        .unwrap();

    let folder = tempfile::tempdir().unwrap();
    let noted = folder.path().join("pid");
    let result = capture(
        &mut shell(&format!(
            "echo started; sleep 30 & echo $! > {}; wait",
            noted.display()
        )),
        Environment::Inherited,
        &[],
        &Policy::tail(LIMIT),
        &cancel,
    );

    assert!(matches!(result, Err(CaptureError::Cancelled)), "{result:?}");
    assert!(started.elapsed() < Duration::from_secs(10));
    assert!(gone(&noted), "the child outlived the cancel");
}

#[test]
fn a_flag_set_before_the_start_cancels_at_once() {
    let cancel = Arc::new(AtomicBool::new(true));
    let result = capture(
        &mut shell("sleep 30"),
        Environment::Inherited,
        &[],
        &Policy::tail(LIMIT),
        &cancel,
    );

    assert!(matches!(result, Err(CaptureError::Cancelled)), "{result:?}");
}

#[test]
fn given_variables_reach_the_program_in_a_clean_environment_too() {
    for environment in [Environment::Clean, Environment::Inherited] {
        let captured = capture(
            &mut shell("echo \"$SYSTEMD_COLORS|$LANG\""),
            environment,
            &[("SYSTEMD_COLORS", "16"), ("LANG", "C.UTF-8")],
            &Policy::tail(LIMIT),
            &Arc::default(),
        )
        .unwrap();

        assert_eq!(captured.output, "16|C.UTF-8\n");
    }
}

#[test]
fn a_capture_error_is_an_error_that_keeps_its_cause() {
    use std::error::Error as _;

    let failed = CaptureError::Failed(std::io::Error::other("no pipe"));

    assert_eq!(failed.to_string(), "no pipe");
    assert_eq!(failed.source().unwrap().to_string(), "no pipe");
    assert!(CaptureError::Missing.source().is_none());
    assert_eq!(CaptureError::Cancelled.to_string(), "cancelled");
}

#[test]
fn a_program_past_its_deadline_is_stopped_and_reports_it() {
    let timeout = Duration::from_millis(200);
    let started = Instant::now();

    let result = capture(
        &mut shell("echo started; sleep 30 & wait"),
        Environment::Inherited,
        &[],
        &Policy::tail(LIMIT).within(timeout),
        &Arc::default(),
    );

    assert!(matches!(result, Err(CaptureError::TimedOut)), "{result:?}");
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[test]
fn without_a_deadline_a_slow_program_runs_to_its_end() {
    let captured = capture(
        &mut shell("sleep 1; echo done"),
        Environment::Inherited,
        &[],
        &Policy::tail(LIMIT),
        &Arc::default(),
    )
    .unwrap();

    assert_eq!(captured.output, "done\n");
}

#[test]
fn failures_name_themselves() {
    assert_eq!(
        CaptureError::Missing.to_string(),
        "the program is not installed"
    );
    assert_eq!(
        CaptureError::TimedOut.to_string(),
        "the program took too long"
    );
}

#[test]
fn an_unstartable_program_is_no_missing_one() {
    let dir = tempfile::tempdir().unwrap();
    let program = dir.path().join("tool");

    std::fs::create_dir_all(&program).unwrap();

    let failed = capture(
        &mut Command::new(&program),
        Environment::Inherited,
        &[],
        &Policy::tail(LIMIT),
        &Arc::default(),
    );

    assert!(matches!(failed, Err(CaptureError::Failed(_))), "{failed:?}");
}

/// The program ends at once, but what it left running holds its output
/// open. The deadline still ends the wait and stops the leftover.
#[test]
fn a_child_that_keeps_the_output_open_ends_at_the_timeout() {
    let folder = tempfile::tempdir().unwrap();
    let noted = folder.path().join("pid");
    let started = Instant::now();

    let late = capture(
        &mut shell(&format!(
            "sleep 30 & echo $! > {}; echo hi",
            noted.display()
        )),
        Environment::Inherited,
        &[],
        &Policy::tail(LIMIT).within(Duration::from_millis(300)),
        &Arc::default(),
    );

    assert!(matches!(late, Err(CaptureError::TimedOut)), "{late:?}");
    assert!(started.elapsed() < Duration::from_secs(10));

    let pid = std::fs::read_to_string(&noted).unwrap();
    let leftover = std::path::Path::new("/proc").join(pid.trim());

    while leftover.exists() && started.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(10));
    }

    assert!(
        !leftover.exists(),
        "the background sleep outlived the timeout"
    );
}

/// Without a deadline a cancel ends the wait for a held open output.
#[test]
fn a_cancel_ends_the_wait_for_a_child_that_keeps_the_output_open() {
    let cancel = Arc::new(AtomicBool::new(false));
    let setter = Arc::clone(&cancel);
    let started = Instant::now();
    let signal = std::thread::Builder::new()
        .name("test-cancel".to_owned())
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            setter.store(true, Ordering::Relaxed);
        })
        .unwrap();

    let cancelled = capture(
        &mut shell("sleep 30 & echo hi"),
        Environment::Inherited,
        &[],
        &Policy::tail(LIMIT),
        &cancel,
    );

    signal.join().unwrap();

    assert!(
        matches!(cancelled, Err(CaptureError::Cancelled)),
        "{cancelled:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(10));
}
