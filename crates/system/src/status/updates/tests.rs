use std::path::{Path, PathBuf};

use tempfile::TempDir;
use toolbox_plan::PackageName;

use super::{
    AurUpdateSource as _, Checkupdates, Paru, PendingUpdate, UpdateSource as _, parse_updates,
};
use crate::ProbeError;
use crate::testing::fake_program;

fn update(name: &str, from: &str, to: &str) -> PendingUpdate {
    PendingUpdate {
        name: PackageName::new(name).unwrap(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

/// A script that prints `stdout`, a harmless complaint and exits with
/// `code`, standing in for checkupdates or paru.
fn script(stdout: &str, code: u8) -> (TempDir, PathBuf) {
    fake_program(stdout, "fake: complaint\n", code)
}

#[test]
fn checkupdates_lines_are_updates() {
    let output = "linux-zen 6.11.1.zen1-1 -> 6.11.2.zen1-1\n\
                  lib32-glibc 2.40+r16-1 -> 2.40+r16-2\n\
                  firefox 1:131.0-1 -> 1:131.0.2-1\n";

    assert_eq!(
        parse_updates(output),
        [
            update("linux-zen", "6.11.1.zen1-1", "6.11.2.zen1-1"),
            update("lib32-glibc", "2.40+r16-1", "2.40+r16-2"),
            update("firefox", "1:131.0-1", "1:131.0.2-1"),
        ]
    );
}

#[test]
fn empty_output_is_no_updates() {
    assert_eq!(parse_updates(""), []);
    assert_eq!(parse_updates("\n  \n"), []);
}

#[test]
fn ignored_aur_updates_are_left_out() {
    let output = "paru-bin 2.0.3-1 -> 2.0.4-1\n\
                  yay 12.3.0-1 -> 12.4.0-1 [ignored]\n";

    assert_eq!(
        parse_updates(output),
        [update("paru-bin", "2.0.3-1", "2.0.4-1")]
    );
}

#[test]
fn other_trailing_text_is_dropped() {
    let output = "foo-git r12.abc-1 -> r13.def-1 (devel)\n";

    assert_eq!(
        parse_updates(output),
        [update("foo-git", "r12.abc-1", "r13.def-1")]
    );
}

#[test]
fn odd_lines_are_skipped() {
    let output = ":: Looking for devel upgrades...\n\
                  warning: something\n\
                  bash 5.2-1 5.3-1\n\
                  bash 5.2-1 ->\n\
                  -bad 1-1 -> 2-1\n\
                     vim   9.1-1   ->   9.2-1   \n";

    assert_eq!(parse_updates(output), [update("vim", "9.1-1", "9.2-1")]);
}

#[test]
fn checkupdates_exit_2_is_no_updates() {
    let (_folder, fake) = script("", 2);

    assert_eq!(Checkupdates::with_program(fake).pending(), Ok(Vec::new()));
}

#[test]
fn checkupdates_exit_0_lists_updates() {
    let (_folder, fake) = script("toolbox-test-up 1.0-1 -> 1.1-1\n", 0);

    assert_eq!(
        Checkupdates::with_program(fake).pending(),
        Ok(vec![update("toolbox-test-up", "1.0-1", "1.1-1")])
    );
}

#[test]
fn checkupdates_exit_1_fails_with_its_complaint() {
    let (_folder, fake) = script("", 1);
    let result = Checkupdates::with_program(fake).pending();

    assert!(
        matches!(&result, Err(ProbeError::Failed(reason)) if reason.contains("fake: complaint")),
        "{result:?}"
    );
}

#[test]
fn a_missing_checkupdates_is_unavailable() {
    let checkupdates = Checkupdates::with_program("toolbox-system-no-such-program");

    assert_eq!(checkupdates.pending(), Err(ProbeError::Unavailable));
}

#[test]
fn paru_exit_1_without_output_is_no_updates() {
    let (_folder, fake) = script("", 1);

    assert_eq!(Paru::with_program(fake).pending_aur(), Ok(Vec::new()));
}

#[test]
fn paru_exit_0_lists_updates_without_ignored_ones() {
    let (_folder, fake) = script("a 1-1 -> 2-1\nb 1-1 -> 2-1 [ignored]\n", 0);

    assert_eq!(
        Paru::with_program(fake).pending_aur(),
        Ok(vec![update("a", "1-1", "2-1")])
    );
}

/// paru also exits with 1 when the AUR cannot be reached.
#[test]
fn paru_exit_1_with_an_error_fails() {
    let (_folder, fake) = fake_program("", "error: failed to connect to aur.archlinux.org\n", 1);
    let result = Paru::with_program(fake).pending_aur();

    assert!(
        matches!(&result, Err(ProbeError::Failed(reason)) if reason.contains("aur.archlinux.org")),
        "{result:?}"
    );
}

#[test]
fn paru_exit_1_with_output_fails() {
    let (_folder, fake) = script("a 1-1 -> 2-1\n", 1);

    assert!(matches!(
        Paru::with_program(fake).pending_aur(),
        Err(ProbeError::Failed(_))
    ));
}

#[test]
fn paru_other_exit_codes_fail() {
    let (_folder, fake) = script("", 3);

    assert!(matches!(
        Paru::with_program(fake).pending_aur(),
        Err(ProbeError::Failed(_))
    ));
}

#[test]
fn a_missing_paru_is_unavailable() {
    let paru = Paru::with_program(Path::new("toolbox-system-no-such-program"));

    assert_eq!(paru.pending_aur(), Err(ProbeError::Unavailable));
}

#[test]
fn only_the_system_paru_refuses_root() {
    assert!(Paru::new().refuse_root);
    assert!(!Paru::with_program("paru").refuse_root);
}
