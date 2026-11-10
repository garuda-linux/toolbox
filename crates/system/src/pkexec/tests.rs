use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use super::{HelperRun, Refusal, ask, reason, refusal};
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
        HelperRun::Failed(
            "the helper ended with exit code 1: garuda-toolbox-helper: no such file".to_owned()
        )
    );
}

/// The answer of a broken or hostile helper never fills the memory.
#[test]
fn an_answer_past_the_limit_fails() {
    assert_eq!(
        asked("0123456789\n0123456789\n", "", 0, 8),
        HelperRun::Failed("the helper's answer is too long".to_owned())
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
        HelperRun::Failed(text) if text.contains("not installed")
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

#[test]
fn the_exit_code_and_pkexecs_words_tell_the_refusals_apart() {
    for (code, stderr, expected) in [
        (Some(0_i32), "", None),
        (Some(1_i32), "garuda-toolbox-helper: cannot answer", None),
        (
            Some(1_i32),
            "Error executing command as another user: Not authorized",
            None,
        ),
        (None, "", None),
        (Some(126_i32), "", Some(Refusal::Dismissed)),
        (
            Some(126_i32),
            "Error executing command as another user: Request dismissed",
            Some(Refusal::Dismissed),
        ),
        (Some(127_i32), "", Some(Refusal::Denied)),
        (
            Some(127_i32),
            "Error executing command as another user: Not authorized\n\nThis incident has been reported.",
            Some(Refusal::Denied),
        ),
        (Some(127_i32), "words nobody knows", Some(Refusal::Broken)),
        (
            Some(127_i32),
            "pkexec must be setuid root",
            Some(Refusal::Broken),
        ),
        (
            Some(127_i32),
            "Error executing command as another user: No authentication agent found.",
            Some(Refusal::NoAgent),
        ),
        (
            Some(127_i32),
            "Error creating textual authentication agent: No controlling terminal",
            Some(Refusal::NoAgent),
        ),
        (
            Some(127_i32),
            "Error getting authority: Could not connect: No such file or directory",
            Some(Refusal::NoAgent),
        ),
        (
            Some(127_i32),
            "Error checking for authorization org.garudalinux.toolbox.read: boom",
            Some(Refusal::NoAgent),
        ),
        (
            Some(127_i32),
            "ERROR: NO SESSION FOR PID 4242",
            Some(Refusal::NoAgent),
        ),
        (
            Some(127_i32),
            "Cannot run program /usr/lib/garuda-toolbox/garuda-toolbox-helper: No such file or directory",
            Some(Refusal::HelperMissing),
        ),
    ] {
        assert_eq!(refusal(code, stderr), expected, "{code:?} {stderr}");
    }
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
        HelperRun::Failed("pkexec cannot run: pkexec must be setuid root".to_owned())
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
