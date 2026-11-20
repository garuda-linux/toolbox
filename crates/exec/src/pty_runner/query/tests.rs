use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_plan::procedure::{Command, Program};

use super::{MAX_QUERY_BYTES, QueryKind, empty_miss, run};
use crate::pty_runner::Environment;

const SHELL: &str = "/bin/sh";

/// `pacman -T -- cmake qt6-tools` in the Garuda Lima VM, which has neither.
const UNMET: &str = include_str!("fixtures/lima/pacman-t-unmet.stdout");
const UNMET_CODE: &str = include_str!("fixtures/lima/pacman-t-unmet.code");
/// `pacman -T -- pacman bash` in the same VM.
const MET: &str = include_str!("fixtures/lima/pacman-t-met.stdout");
const MET_CODE: &str = include_str!("fixtures/lima/pacman-t-met.code");

fn query(command: &Command, cancel: &Arc<AtomicBool>) -> io::Result<Vec<String>> {
    run(
        SHELL,
        command,
        QueryKind::Listing,
        Environment::Clean,
        cancel,
    )
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
    let error = run(
        SHELL,
        &command,
        QueryKind::Listing,
        Environment::Inherited,
        &Arc::default(),
    )
    .unwrap_err();

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
        QueryKind::Listing,
        Environment::Clean,
        &Arc::default(),
    )
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::NotFound);
}

fn deptest(command: &Command) -> io::Result<Vec<String>> {
    run(
        SHELL,
        command,
        QueryKind::Deptest,
        Environment::Clean,
        &Arc::default(),
    )
}

fn printing(output: &str, code: &str) -> Command {
    script(&format!("printf '%s' '{output}'; exit {}", code.trim()))
}

#[test]
fn a_deptest_answers_its_unmet_names_with_code_127() {
    assert_eq!(
        deptest(&printing(UNMET, UNMET_CODE)).unwrap(),
        ["cmake", "qt6-tools"]
    );
}

#[test]
fn a_deptest_with_everything_met_answers_nothing() {
    assert_eq!(
        deptest(&printing(MET, MET_CODE)).unwrap(),
        Vec::<String>::new()
    );
}

/// pacman ends with 127 only when it names what is missing.
#[test]
fn a_deptest_with_code_127_and_no_name_fails() {
    deptest(&script("exit 127")).unwrap_err();
    deptest(&script("exit 1")).unwrap_err();
}

#[test]
fn code_127_is_no_answer_of_any_other_query() {
    query(&printing(UNMET, UNMET_CODE), &Arc::default()).unwrap_err();
}
