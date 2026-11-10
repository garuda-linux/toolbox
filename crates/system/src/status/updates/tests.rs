use std::path::{Path, PathBuf};

use tempfile::TempDir;
use toolbox_plan::PackageName;

use super::{
    AurUpdateSource as _, Checkupdates, Paru, PendingUpdate, UpdateSource as _, parse_updates,
};
use crate::ProbeError;
use crate::test_support::fake_program;

/// `checkupdates --nocolor` in the Lima VM, one pending update.
const CHECKUPDATES: &str = include_str!("fixtures/checkupdates.stdout");
const CHECKUPDATES_CODE: &str = include_str!("fixtures/checkupdates.code");
/// `checkupdates --nocolor` on an installed Garuda.
const INSTALLED_CHECKUPDATES: &str = include_str!("fixtures/installed-checkupdates.stdout");
/// `paru -Qua` in the Lima VM with one pending AUR update.
const PARU_QUA: &str = include_str!("fixtures/paru-qua.stdout");
const PARU_QUA_CODE: &str = include_str!("fixtures/paru-qua.code");
/// The same update while `IgnorePkg` holds it back.
const PARU_QUA_IGNORED: &str = include_str!("fixtures/paru-qua-ignored.stdout");
const PARU_QUA_IGNORED_CODE: &str = include_str!("fixtures/paru-qua-ignored.code");
/// `paru -Qua` on an installed Garuda with nothing pending printed nothing.
const INSTALLED_PARU_QUA_CODE: &str = include_str!("fixtures/installed-paru-qua.code");

fn code(captured: &str) -> u8 {
    captured.trim().parse().unwrap()
}

fn update(name: &str, from: &str, to: &str) -> PendingUpdate {
    PendingUpdate {
        name: PackageName::new(name).unwrap(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

fn htop() -> PendingUpdate {
    update("htop", "3.5.2-1", "3.5.3-1")
}

/// A script that prints `stdout`, a harmless complaint and exits with
/// `code`, standing in for checkupdates or paru.
fn script(stdout: &str, code: u8) -> (TempDir, PathBuf) {
    fake_program(stdout, "fake: complaint\n", code)
}

#[test]
fn checkupdates_lines_are_updates() {
    assert_eq!(
        parse_updates(INSTALLED_CHECKUPDATES),
        [
            update("7zip", "26.05-1", "26.06-1"),
            update("fastfetch", "2.70.0-1", "2.71.0-1"),
            update("garuda-hooks", "2.16.0-2", "2.16.1-1"),
            update("intel-gmmlib", "22.10.0-1", "22.10.2-1"),
            update("inxi", "3.3.41.1-1", "3.3.41.1-2"),
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
    assert_eq!(
        parse_updates(PARU_QUA),
        [update("cbonsai", "1.4.0-1", "1.4.2-1")]
    );
    assert_eq!(parse_updates(PARU_QUA_IGNORED), []);
}

/// The Lima update with ` (devel)` appended to its line.
#[test]
fn other_trailing_text_is_dropped() {
    let output = CHECKUPDATES.replacen('\n', " (devel)\n", 1);

    assert_eq!(parse_updates(&output), [htop()]);
}

/// Made-up broken lines in front of the Lima update, whose words are
/// padded with more spaces.
#[test]
fn odd_lines_are_skipped() {
    let output = format!(
        ":: Looking for devel upgrades...\n\
         warning: something\n\
         bash 5.2-1 5.3-1\n\
         bash 5.2-1 ->\n\
         -bad 1-1 -> 2-1\n   {}",
        CHECKUPDATES.replace(' ', "   ")
    );

    assert_eq!(parse_updates(&output), [htop()]);
}

#[test]
fn checkupdates_exit_2_is_no_updates() {
    let (_folder, fake) = script("", 2);

    assert_eq!(Checkupdates::with_program(fake).pending(), Ok(Vec::new()));
}

#[test]
fn checkupdates_exit_0_lists_updates() {
    let (_folder, fake) = script(CHECKUPDATES, code(CHECKUPDATES_CODE));

    assert_eq!(Checkupdates::with_program(fake).pending(), Ok(vec![htop()]));
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
    let (_folder, fake) = fake_program("", "", code(INSTALLED_PARU_QUA_CODE));

    assert_eq!(Paru::with_program(fake).pending_aur(), Ok(Vec::new()));
}

#[test]
fn paru_exit_0_lists_updates_without_ignored_ones() {
    let (_folder, fake) = script(PARU_QUA, code(PARU_QUA_CODE));
    let (_other, ignored) = script(PARU_QUA_IGNORED, code(PARU_QUA_IGNORED_CODE));

    assert_eq!(
        Paru::with_program(fake).pending_aur(),
        Ok(vec![update("cbonsai", "1.4.0-1", "1.4.2-1")])
    );
    assert_eq!(Paru::with_program(ignored).pending_aur(), Ok(Vec::new()));
}

/// paru also exits with 1 when the AUR cannot be reached. No such run was
/// captured, so the error line is made up.
#[test]
fn paru_exit_1_with_an_error_fails() {
    let (_folder, fake) = fake_program("", "error: failed to connect to aur.archlinux.org\n", 1);
    let result = Paru::with_program(fake).pending_aur();

    assert!(
        matches!(&result, Err(ProbeError::Failed(reason)) if reason.contains("aur.archlinux.org")),
        "{result:?}"
    );
}

/// The Lima output with the exit code changed to 1.
#[test]
fn paru_exit_1_with_output_fails() {
    let (_folder, fake) = script(PARU_QUA, 1);

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
