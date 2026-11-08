use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_plan::procedure::{Command, Program};

use super::{MAX_QUERY_BYTES, empty_miss, run};
use crate::pty_runner::Environment;

const SHELL: &str = "/bin/sh";

fn query(command: &Command, cancel: &Arc<AtomicBool>) -> io::Result<Vec<String>> {
    run(SHELL, command, Environment::Clean, cancel)
}

fn script(script: &str) -> Command {
    Command::new(Program::Pacman).args(["-c", script])
}

#[test]
fn only_code_one_with_empty_output_is_a_miss() {
    assert!(empty_miss(Some(1_i32), true));
    assert!(!empty_miss(Some(1_i32), false));
    assert!(!empty_miss(Some(2_i32), true));
    assert!(!empty_miss(Some(0_i32), true));
    assert!(!empty_miss(None, true));
}

#[test]
fn code_one_with_an_error_message_is_a_failure_not_a_miss() {
    let failed = query(
        &script("echo 'error: no operation specified' >&2; exit 1"),
        &Arc::default(),
    );

    assert!(
        failed.is_err(),
        "an error read as an empty answer: {failed:?}"
    );
}

#[test]
fn the_output_lines_are_the_answer() {
    assert_eq!(
        query(&script("printf 'a\\n\\nb\\n'"), &Arc::default()).unwrap(),
        ["a", "b"]
    );
    assert_eq!(
        query(&script("exit 1"), &Arc::default()).unwrap(),
        Vec::<String>::new()
    );
}

/// `SYSTEMD_COLORS=0` and the like must reach the program, not only the
/// steps that run in a terminal.
#[test]
fn the_commands_variables_reach_the_program() {
    let command = script("echo \"$GREETING $LANG\"").env("GREETING", "hello");

    assert_eq!(query(&command, &Arc::default()).unwrap(), ["hello C.UTF-8"]);
}

#[test]
fn a_failure_keeps_the_programs_words() {
    let error = query(
        &script("echo 'database is locked' >&2; exit 2"),
        &Arc::default(),
    )
    .unwrap_err();

    assert!(error.to_string().contains("database is locked"), "{error}");
    assert!(error.to_string().contains("exit code 2"), "{error}");
}

#[test]
fn an_answer_past_the_cap_fails() {
    let command = script(&format!(
        "head -c {} /dev/zero | tr '\\0' 'a'",
        MAX_QUERY_BYTES + 1
    ));

    // Inherited, so `head` is found on a system without `/usr/bin`.
    let error = run(SHELL, &command, Environment::Inherited, &Arc::default()).unwrap_err();

    assert!(error.to_string().contains("more than"), "{error}");
}

#[test]
fn a_stop_ends_the_query() {
    let error = query(&script("sleep 30"), &Arc::new(AtomicBool::new(true))).unwrap_err();

    assert!(error.to_string().contains("cancelled"), "{error}");
}

#[test]
fn a_missing_program_is_not_found() {
    let error = run(
        "/nonexistent/pacman",
        &script(""),
        Environment::Clean,
        &Arc::default(),
    )
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::NotFound);
}
