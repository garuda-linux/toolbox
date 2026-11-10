use std::path::PathBuf;

use tempfile::TempDir;

use super::{GarudaHealth, HealthIssue, HealthLevel, HealthSource as _, parse_health};
use crate::ProbeError;
use crate::test_support::fake_program;

/// `garuda-health --json` 1.11.1 in the Lima VM, which exited with 1.
const REPORT: &str = include_str!("fixtures/garuda-health.json");

/// The Lima report found no CRITICAL or HIGH issue. These two are added in
/// front, in the order garuda-health prints its levels.
fn with_critical_and_high() -> String {
    REPORT.replacen(
        r#""LOW": ["#,
        r#""CRITICAL": [{"description": "Core garuda linux packages are missing", "fix_available": true}],
  "HIGH": [{"description": "Important garuda linux packages are missing", "fix_available": true}],
  "LOW": ["#,
        1,
    )
}

fn issue(level: HealthLevel, message: &str, fix_available: bool) -> HealthIssue {
    HealthIssue {
        level,
        message: message.to_owned(),
        fix_available,
    }
}

fn script(stdout: &str, code: u8) -> (TempDir, PathBuf) {
    fake_program(stdout, "health: complaint\n", code)
}

#[test]
fn a_real_report_is_read_most_severe_first() {
    assert_eq!(
        parse_health(REPORT),
        Ok(vec![
            issue(
                HealthLevel::Low,
                "Failed systemd services found: cloud-init-main.service, toolbox-fixture-failed.service",
                false
            ),
            issue(
                HealthLevel::Low,
                "\"Garuda\" is not the current bootloader: https://wiki.garudalinux.org/why-garuda-bootloader",
                false
            ),
            issue(
                HealthLevel::Info,
                "Optional garuda linux utilities are missing: garuda-toolbox, garuda-system-maintenance",
                true
            ),
            issue(
                HealthLevel::Info,
                "System is not using PipeWire for audio",
                true
            ),
            issue(
                HealthLevel::Info,
                "A reboot is pending (update applied since last reboot)",
                false
            ),
            issue(
                HealthLevel::Info,
                "Snapper (automatic btrfs system restore) is not using the official Garuda Linux configuration",
                true
            ),
        ])
    );
}

#[test]
fn critical_and_high_come_before_low() {
    let levels: Vec<HealthLevel> = parse_health(&with_critical_and_high())
        .unwrap()
        .into_iter()
        .map(|issue| issue.level)
        .collect();

    assert_eq!(
        levels,
        [
            HealthLevel::Critical,
            HealthLevel::High,
            HealthLevel::Low,
            HealthLevel::Low,
            HealthLevel::Info,
            HealthLevel::Info,
            HealthLevel::Info,
            HealthLevel::Info,
        ]
    );
    assert!(HealthLevel::Critical > HealthLevel::High);
    assert!(HealthLevel::High > HealthLevel::Low);
    assert!(HealthLevel::Low > HealthLevel::Info);
}

#[test]
fn missing_levels_and_empty_output_are_healthy() {
    assert_eq!(parse_health("{}"), Ok(Vec::new()));
    assert_eq!(parse_health(r#"{"LOW": []}"#), Ok(Vec::new()));
    assert_eq!(parse_health(""), Ok(Vec::new()));
    assert_eq!(parse_health("  \n"), Ok(Vec::new()));
}

#[test]
fn entries_without_description_are_skipped() {
    let report =
        r#"{"HIGH": [{"fix_available": true}, {"description": "  "}, {"description": "real"}]}"#;

    assert_eq!(
        parse_health(report),
        Ok(vec![issue(HealthLevel::High, "real", false)])
    );
}

#[test]
fn unknown_levels_are_ignored() {
    let report = r#"{"ERROR": [{"description": "x"}], "WARNING": [{"description": "x"}], "INFO": [{"description": "y"}]}"#;

    assert_eq!(
        parse_health(report),
        Ok(vec![issue(HealthLevel::Info, "y", false)])
    );
}

#[test]
fn text_that_is_no_report_fails() {
    for text in ["not json", "[]", r#"{"LOW": "broken"}"#, r#"{"LOW": [1]}"#] {
        assert!(
            matches!(parse_health(text), Err(ProbeError::Failed(_))),
            "{text}"
        );
    }
}

/// It exits with an error when it finds issues, the report is on stdout
/// all the same.
#[test]
fn a_report_counts_whatever_the_exit_code() {
    let (_folder, fake) = script(r#"{"CRITICAL": [{"description": "bad"}]}"#, 1);

    assert_eq!(
        GarudaHealth::with_program(fake).issues(),
        Ok(vec![issue(HealthLevel::Critical, "bad", false)])
    );
}

#[test]
fn a_failure_without_report_fails() {
    let (_folder, fake) = script("", 1);
    let result = GarudaHealth::with_program(fake).issues();

    assert!(
        matches!(&result, Err(ProbeError::Failed(reason)) if reason.contains("health: complaint")),
        "{result:?}"
    );
}

#[test]
fn a_missing_garuda_health_is_unavailable() {
    let health = GarudaHealth::with_program("toolbox-system-no-such-program");

    assert_eq!(health.issues(), Err(ProbeError::Unavailable));
}
