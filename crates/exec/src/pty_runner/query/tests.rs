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

/// `pacman -Qdtq -- cowsay` and the other answers of the Lima VM, captured
/// as pacman printed them.
const ONE_ORPHAN_OUT: &str = include_str!("fixtures/lima/pacman-qdtq-one-orphan.stdout");
const ONE_ORPHAN_ERR: &str = include_str!("fixtures/lima/pacman-qdtq-one-orphan.stderr");
const ONE_ORPHAN_CODE: &str = include_str!("fixtures/lima/pacman-qdtq-one-orphan.code");
const NOT_AN_ORPHAN_OUT: &str = include_str!("fixtures/lima/pacman-qdtq-not-an-orphan.stdout");
const NOT_AN_ORPHAN_ERR: &str = include_str!("fixtures/lima/pacman-qdtq-not-an-orphan.stderr");
const NOT_AN_ORPHAN_CODE: &str = include_str!("fixtures/lima/pacman-qdtq-not-an-orphan.code");
const ORPHAN_AND_NOT_OUT: &str = include_str!("fixtures/lima/pacman-qdtq-orphan-and-not.stdout");
const ORPHAN_AND_NOT_ERR: &str = include_str!("fixtures/lima/pacman-qdtq-orphan-and-not.stderr");
const ORPHAN_AND_NOT_CODE: &str = include_str!("fixtures/lima/pacman-qdtq-orphan-and-not.code");
const NOT_INSTALLED_OUT: &str = include_str!("fixtures/lima/pacman-qdtq-not-installed.stdout");
const NOT_INSTALLED_ERR: &str = include_str!("fixtures/lima/pacman-qdtq-not-installed.stderr");
const NOT_INSTALLED_CODE: &str = include_str!("fixtures/lima/pacman-qdtq-not-installed.code");
const ORPHAN_AND_NOT_INSTALLED_OUT: &str =
    include_str!("fixtures/lima/pacman-qdtq-orphan-and-not-installed.stdout");
const ORPHAN_AND_NOT_INSTALLED_ERR: &str =
    include_str!("fixtures/lima/pacman-qdtq-orphan-and-not-installed.stderr");
const ORPHAN_AND_NOT_INSTALLED_CODE: &str =
    include_str!("fixtures/lima/pacman-qdtq-orphan-and-not-installed.code");
const OPTIONALLY_REQUIRED_OUT: &str =
    include_str!("fixtures/lima/pacman-qdtq-optionally-required.stdout");
const OPTIONALLY_REQUIRED_ERR: &str =
    include_str!("fixtures/lima/pacman-qdtq-optionally-required.stderr");
const OPTIONALLY_REQUIRED_CODE: &str =
    include_str!("fixtures/lima/pacman-qdtq-optionally-required.code");
const ORPHAN_AND_OPTIONAL_OUT: &str =
    include_str!("fixtures/lima/pacman-qdtq-orphan-and-optional.stdout");
const ORPHAN_AND_OPTIONAL_ERR: &str =
    include_str!("fixtures/lima/pacman-qdtq-orphan-and-optional.stderr");
const ORPHAN_AND_OPTIONAL_CODE: &str =
    include_str!("fixtures/lima/pacman-qdtq-orphan-and-optional.code");
const AFTER_REMOVAL_OUT: &str = include_str!("fixtures/lima/pacman-qdtq-after-removal.stdout");
const AFTER_REMOVAL_ERR: &str = include_str!("fixtures/lima/pacman-qdtq-after-removal.stderr");
const AFTER_REMOVAL_CODE: &str = include_str!("fixtures/lima/pacman-qdtq-after-removal.code");

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

#[test]
fn a_deptest_with_code_127_and_no_name_fails() {
    deptest(&script("exit 127")).unwrap_err();
    deptest(&script("exit 1")).unwrap_err();
}

#[test]
fn code_127_is_no_answer_of_any_other_query() {
    query(&printing(UNMET, UNMET_CODE), &Arc::default()).unwrap_err();
}

/// A stand-in that prints `stdout` and `stderr` and exits with `code`,
/// asked about `asked` as `pacman -Qdtq --` gets them.
fn captured(stdout: &str, stderr: &str, code: &str, asked: &[&str]) -> Command {
    let script = format!(
        "printf '%s' {} ; printf '%s' {} >&2 ; exit {}",
        quoted(stdout),
        quoted(stderr),
        code.trim()
    );

    Command::new(Program::Pacman)
        .args(["-c", &script, "pacman", "-Qdtq", "--"])
        .args(asked.iter().copied())
}

fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

fn orphans(command: &Command) -> io::Result<Vec<String>> {
    run(
        SHELL,
        command,
        QueryKind::Orphans,
        Environment::Clean,
        &Arc::default(),
    )
}

#[test]
fn an_orphan_is_the_answer() {
    assert_eq!(
        orphans(&captured(
            ONE_ORPHAN_OUT,
            ONE_ORPHAN_ERR,
            ONE_ORPHAN_CODE,
            &["cowsay"]
        ))
        .unwrap(),
        ["cowsay"]
    );
}

#[test]
fn a_name_that_is_required_is_no_answer_with_code_one() {
    assert_eq!(
        orphans(&captured(
            NOT_AN_ORPHAN_OUT,
            NOT_AN_ORPHAN_ERR,
            NOT_AN_ORPHAN_CODE,
            &["bash"]
        ))
        .unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn the_orphans_among_several_names_are_printed_with_code_zero() {
    assert_eq!(
        orphans(&captured(
            ORPHAN_AND_NOT_OUT,
            ORPHAN_AND_NOT_ERR,
            ORPHAN_AND_NOT_CODE,
            &["cowsay", "bash"]
        ))
        .unwrap(),
        ["cowsay"]
    );
}

#[test]
fn a_name_that_is_not_installed_is_no_answer_with_code_one() {
    assert_eq!(
        orphans(&captured(
            NOT_INSTALLED_OUT,
            NOT_INSTALLED_ERR,
            NOT_INSTALLED_CODE,
            &["no-such-package-xyz"]
        ))
        .unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn the_orphans_among_names_with_one_not_installed_are_printed_with_code_one() {
    assert_eq!(
        orphans(&captured(
            ORPHAN_AND_NOT_INSTALLED_OUT,
            ORPHAN_AND_NOT_INSTALLED_ERR,
            ORPHAN_AND_NOT_INSTALLED_CODE,
            &["cowsay", "no-such-package-xyz"]
        ))
        .unwrap(),
        ["cowsay"]
    );
}

#[test]
fn an_optional_requirement_keeps_a_name_out_with_code_one() {
    assert_eq!(
        orphans(&captured(
            OPTIONALLY_REQUIRED_OUT,
            OPTIONALLY_REQUIRED_ERR,
            OPTIONALLY_REQUIRED_CODE,
            &["tree"]
        ))
        .unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn an_orphan_beside_an_optional_requirement_is_printed() {
    assert_eq!(
        orphans(&captured(
            ORPHAN_AND_OPTIONAL_OUT,
            ORPHAN_AND_OPTIONAL_ERR,
            ORPHAN_AND_OPTIONAL_CODE,
            &["cowsay", "tree"]
        ))
        .unwrap(),
        ["cowsay"]
    );
}

#[test]
fn a_name_removed_meanwhile_is_no_answer_with_code_one() {
    assert_eq!(
        orphans(&captured(
            AFTER_REMOVAL_OUT,
            AFTER_REMOVAL_ERR,
            AFTER_REMOVAL_CODE,
            &["cowsay"]
        ))
        .unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn printed_names_without_a_not_found_error_fail_with_code_one() {
    orphans(&captured(ONE_ORPHAN_OUT, "", "1", &["cowsay"])).unwrap_err();
}

#[test]
fn a_not_found_error_for_a_name_not_asked_fails() {
    orphans(&captured(
        "",
        "error: package 'bash' was not found\n",
        "1",
        &["cowsay"],
    ))
    .unwrap_err();
}

#[test]
fn any_other_error_fails_the_orphan_query() {
    orphans(&captured(
        "",
        "error: failed to open the database\n",
        "1",
        &["cowsay"],
    ))
    .unwrap_err();
}

#[test]
fn a_warning_with_code_zero_fails_the_orphan_query() {
    orphans(&captured(ONE_ORPHAN_OUT, "warning: x\n", "0", &["cowsay"])).unwrap_err();
}

#[test]
fn other_codes_fail_the_orphan_query() {
    orphans(&captured(ONE_ORPHAN_OUT, ONE_ORPHAN_ERR, "2", &["cowsay"])).unwrap_err();
    orphans(&captured("", "", "127", &["cowsay"])).unwrap_err();
}
