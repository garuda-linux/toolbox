//! The diagnostics collectors against the Arch test image
//! (`ci/arch/Containerfile`): real `systemd-analyze` and `journalctl` without
//! systemd, the real pacman log, no Garuda tools. They run only with
//! `TOOLBOX_IT_ARCH=1`, as root and with `/etc/arch-release`, and return at
//! once elsewhere. Run them with
//! `scripts/arch-it.sh -p toolbox-system --test arch_diagnostics`.

mod common;

use std::error::Error;
use std::process::Command;
use std::sync::Arc;

use toolbox_plan::protocol::ReadKind;
use toolbox_state::diagnostics::{Collector, Outcome};
use toolbox_system::{DiagnosticSource as _, HelperReads, SystemDiagnostics};

type TestResult = Result<(), Box<dyn Error>>;

/// The helper is tested in its own crate. Here a read through it would
/// mean the collector asked for root as root.
struct NoHelper;

impl HelperReads for NoHelper {
    fn read_command(&self, _kind: ReadKind) -> Command {
        Command::new("/nonexistent/garuda-toolbox-helper")
    }
}

fn collect(collector: Collector) -> Result<Outcome, Box<dyn Error>> {
    SystemDiagnostics::system(Arc::new(NoHelper))
        .collect(collector, &Arc::default())
        .ok_or_else(|| "the collector was cancelled".into())
}

#[test]
fn garuda_tools_are_not_available_on_plain_arch() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    assert_eq!(collect(Collector::Inxi)?, Outcome::Unavailable);
    assert_eq!(collect(Collector::Health)?, Outcome::Unavailable);
    Ok(())
}

#[test]
fn systemd_analyze_without_systemd_fails_with_its_own_words() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    match collect(Collector::SystemdAnalyze)? {
        Outcome::Failed(reason) => assert_ne!(reason.trim(), ""),
        // Some versions print a partial answer before they give up.
        Outcome::Report(text) => assert!(text.contains("Exit code: "), "{text}"),
        Outcome::Unavailable => panic!("systemd is part of the image"),
        Outcome::Refused(_) => panic!("nothing asks for a password here"),
    }

    Ok(())
}

#[test]
fn the_journal_without_journald_gives_a_clean_answer() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    match collect(Collector::Journal)? {
        // The report may hold colours. What is copied and uploaded may not.
        Outcome::Report(text) => assert!(!toolbox_pty::strip_ansi(&text).contains('\u{1b}')),
        Outcome::Failed(reason) => {
            assert_ne!(reason.trim(), "");
            assert!(!reason.contains('\u{1b}'), "a reason with colour codes");
            assert!(!reason.contains("helper"), "root asked for root: {reason}");
        }
        Outcome::Unavailable => panic!("journalctl is part of the image"),
        Outcome::Refused(_) => panic!("nothing asks for a password here"),
    }

    Ok(())
}

#[test]
fn the_real_pacman_log_is_read_from_its_last_run() -> TestResult {
    if !common::in_test_image() {
        return Ok(());
    }

    let Outcome::Report(text) = collect(Collector::PacmanLog)? else {
        panic!("the image's pacman log was not read");
    };

    let is_run = |line: &str| {
        line.split_once("PACMAN")
            .is_some_and(|(_, rest)| rest.contains("pacman"))
    };

    assert!(text.lines().next().is_some_and(is_run), "{text}");
    assert_eq!(
        text.lines().filter(|line| is_run(line)).count(),
        1,
        "the part starts before the last run"
    );
    Ok(())
}
