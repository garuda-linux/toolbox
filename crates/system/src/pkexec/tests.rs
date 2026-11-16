use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use super::{HelperError, HelperRun, Refusal, ask, reason, refusal};
use crate::test_support::fake_program;

const LIMIT: usize = 1 << 20;

fn asked(stdout: &str, stderr: &str, code: u8, limit: usize) -> HelperRun {
    let (_folder, program) = fake_program(stdout, stderr, code);

    ask(&mut Command::new(program), limit, &Arc::default())
}

#[test]
fn a_helper_run_is_answered_refused_or_failed_with_its_words() {
    assert_eq!(
        asked("{\"a\":1}\n", "", 0, LIMIT),
        HelperRun::Answered("{\"a\":1}\n".to_owned())
    );
    assert_eq!(
        asked("", "", 126, LIMIT),
        HelperRun::Refused(Refusal::Dismissed)
    );
    assert_eq!(
        asked("", "garuda-toolbox-helper: no such file", 1, LIMIT),
        HelperRun::Failed(HelperError::Ended {
            code: Some(1_i32),
            words: "garuda-toolbox-helper: no such file".to_owned(),
        })
    );
}

/// The answer of a broken or hostile helper never fills the memory.
#[test]
fn an_answer_past_the_limit_fails() {
    assert_eq!(
        asked("0123456789\n0123456789\n", "", 0, 8),
        HelperRun::Failed(HelperError::TooLong)
    );
}

#[test]
fn a_missing_helper_and_a_cancel_are_told_apart() {
    assert!(matches!(
        ask(
            &mut Command::new("/nonexistent/pkexec"),
            LIMIT,
            &Arc::default()
        ),
        HelperRun::Failed(HelperError::Missing)
    ));
    assert_eq!(
        ask(
            Command::new("/bin/sh").args(["-c", "sleep 30"]),
            LIMIT,
            &Arc::new(AtomicBool::new(true))
        ),
        HelperRun::Cancelled
    );
}

/// Rows without program text, so only the exit code decides. The rows with
/// pkexec's words are the fixtures below.
#[test]
fn the_exit_code_alone_decides_without_words() {
    for (code, expected) in [
        (Some(0_i32), None),
        (None, None),
        (Some(126_i32), Some(Refusal::Dismissed)),
        (Some(127_i32), Some(Refusal::Denied)),
    ] {
        assert_eq!(refusal(code, ""), expected, "{code:?}");
    }
}

#[test]
fn a_dismissed_dialog_is_dismissed() {
    let typed = "Error executing command as another user: Request dismissed\n";

    assert_eq!(refusal(Some(126_i32), typed), Some(Refusal::Dismissed));
}

/// pkexec's own answers, captured in a Garuda VM and the Garuda desktop VM.
/// Plasma's agent answers a Cancel like a wrong password, not with 126.
#[test]
fn real_pkexec_refusals_are_told_apart() {
    for (code, stderr, expected) in [
        (
            include_str!("fixtures/no-agent.code"),
            include_str!("fixtures/no-agent.stderr"),
            Refusal::NoAgent,
        ),
        (
            include_str!("fixtures/missing-program.code"),
            include_str!("fixtures/missing-program.stderr"),
            Refusal::HelperMissing,
        ),
        (
            include_str!("fixtures/not-authorized.code"),
            include_str!("fixtures/not-authorized.stderr"),
            Refusal::Denied,
        ),
        (
            include_str!("fixtures/not-setuid.code"),
            include_str!("fixtures/not-setuid.stderr"),
            Refusal::Broken,
        ),
        (
            include_str!("fixtures/polkit-down.code"),
            include_str!("fixtures/polkit-down.stderr"),
            Refusal::NoAgent,
        ),
        (
            include_str!("fixtures/dismissed.code"),
            include_str!("fixtures/dismissed.stderr"),
            Refusal::Denied,
        ),
        (
            include_str!("fixtures/no-agent-found.code"),
            include_str!("fixtures/no-agent-found.stderr"),
            Refusal::NoAgent,
        ),
        (
            include_str!("fixtures/dead-parent.code"),
            include_str!("fixtures/dead-parent.stderr"),
            Refusal::Broken,
        ),
    ] {
        let code = code.trim().parse().ok();

        assert_eq!(refusal(code, stderr), Some(expected), "{code:?} {stderr}");
    }
}

#[test]
fn every_refusal_has_its_own_code() {
    assert_eq!(
        [
            Refusal::Dismissed,
            Refusal::Denied,
            Refusal::NoAgent,
            Refusal::HelperMissing,
            Refusal::Broken
        ]
        .map(Refusal::code),
        [
            "dismissed",
            "denied",
            "no-agent",
            "helper-missing",
            "broken"
        ]
    );
}

#[test]
fn a_broken_pkexec_tells_its_own_words() {
    assert_eq!(
        reason("pkexec must be setuid root\n"),
        "pkexec must be setuid root"
    );
    assert_eq!(
        reason("\nError executing command as another user: boom\n\nmore"),
        "Error executing command as another user: boom"
    );
}

#[test]
fn a_broken_pkexec_read_fails_with_its_words() {
    let run = asked("", "pkexec must be setuid root", 127, 1024);

    assert_eq!(
        run,
        HelperRun::Failed(HelperError::Pkexec {
            words: "pkexec must be setuid root".to_owned()
        })
    );
}

#[test]
fn only_a_missing_dialog_or_helper_is_no_matter_of_rights() {
    assert!(Refusal::Dismissed.not_authorized());
    assert!(Refusal::Denied.not_authorized());
    assert!(!Refusal::NoAgent.not_authorized());
    assert!(!Refusal::HelperMissing.not_authorized());
    assert!(!Refusal::Broken.not_authorized());
}
