use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use toolbox_plan::procedure::{Command, Program};
use toolbox_plan::protocol::{Event, Failure, Outcome};
use toolbox_plan::{ConfigText, StepId};

use super::{CHECK_DEADLINE, policy, report, run};
use crate::capture::CaptureError;
use crate::pty_runner::Environment;
use crate::test_support::printed;

const SHELL: &str = "/bin/sh";
const FIRST: &str = "Defaults env_reset";

/// A text that fills a pipe more than once, first line [`FIRST`].
fn long_text() -> ConfigText {
    let filler = "# a comment line that fills the pipe faster than a short one\n".repeat(2000);
    let text = ConfigText::new(format!("{FIRST}\n{filler}")).unwrap();

    assert!(text.as_str().len() > 64_usize * 1024);
    text
}

fn visudo(script: &str) -> Command {
    Command::new(Program::Visudo).args(["-c", script])
}

fn checked(script: &str, text: &ConfigText, deadline: Duration) -> (Outcome, String) {
    let command = visudo(script);
    let result = run(
        SHELL,
        &command,
        Environment::Clean,
        &policy(text, deadline),
        &Arc::default(),
    );

    let mut events: Vec<Event> = Vec::new();
    let outcome = report(StepId::new(2), &command, deadline, result, &mut events);

    (outcome, printed(&events))
}

/// Shell builtins only: the clean `PATH` has no `wc` or `cat` on NixOS.
#[test]
fn a_long_text_reaches_the_program_and_its_words_arrive_unscrubbed() {
    let script = "read -r first; n=1; while read -r line; do n=$((n+1)); done; \
                  echo \"read $n lines, first: $first, home: ${HOME-none}\"; \
                  echo 'visudo: syntax error near line 2' >&2; exit 1";

    let (outcome, said) = checked(script, &long_text(), CHECK_DEADLINE);

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::ExitCode { code: 1_i32 },
            cleanup_failed: Vec::new()
        }
    );
    assert_eq!(
        said,
        format!(
            "read 2001 lines, first: {FIRST}, home: none\r\nvisudo: syntax error near line 2\r\n"
        )
    );
}

#[test]
fn a_text_the_program_accepts_passes() {
    let (outcome, said) = checked(
        "while read -r line; do :; done; echo 'parsed OK'",
        &long_text(),
        CHECK_DEADLINE,
    );

    assert_eq!(outcome, Outcome::Succeeded);
    assert_eq!(said, "parsed OK\r\n");
}

/// It ends before reading a byte, so the broken pipe adds nothing.
#[test]
fn a_program_that_reads_nothing_ends_the_check_with_its_answer() {
    let (outcome, said) = checked(
        "echo 'no input wanted' >&2; exit 3",
        &long_text(),
        CHECK_DEADLINE,
    );

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::ExitCode { code: 3_i32 },
            cleanup_failed: Vec::new()
        }
    );
    assert_eq!(said, "no input wanted\r\n");
}

#[test]
fn a_program_that_never_reads_is_stopped_at_the_deadline() {
    let started = Instant::now();

    let (outcome, said) = checked(
        "while :; do :; done",
        &long_text(),
        Duration::from_millis(300),
    );

    assert!(started.elapsed() < Duration::from_secs(20));
    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown,
            cleanup_failed: Vec::new()
        }
    );
    assert_eq!(
        said,
        "visudo: it did not end within 0 seconds and was stopped\r\n"
    );
}

#[test]
fn a_missing_checker_fails_the_step() {
    let command = visudo("");
    let result = run(
        "/nonexistent/visudo",
        &command,
        Environment::Clean,
        &policy(&long_text(), CHECK_DEADLINE),
        &Arc::default(),
    );

    assert!(matches!(result, Err(CaptureError::Missing)), "{result:?}");

    let mut events: Vec<Event> = Vec::new();
    let outcome = report(
        StepId::new(2),
        &command,
        CHECK_DEADLINE,
        result,
        &mut events,
    );

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown,
            cleanup_failed: Vec::new()
        }
    );
    assert!(printed(&events).contains("/usr/bin/visudo"), "{events:?}");
}

#[test]
fn a_stop_before_the_start_cancels_the_check() {
    let command = visudo("while :; do :; done");
    let result = run(
        SHELL,
        &command,
        Environment::Clean,
        &policy(&long_text(), CHECK_DEADLINE),
        &Arc::new(AtomicBool::new(true)),
    );

    let mut events: Vec<Event> = Vec::new();

    assert_eq!(
        report(
            StepId::new(2),
            &command,
            CHECK_DEADLINE,
            result,
            &mut events
        ),
        Outcome::Cancelled
    );
}
