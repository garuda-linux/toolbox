use std::path::PathBuf;

use tempfile::TempDir;

use super::{GarudaHealth, HealthIssue, HealthLevel, HealthSource as _, parse_health};
use crate::ProbeError;
use crate::testing::fake_program;

/// The shape `garuda-health --json` prints.
const REPORT: &str = r#"{
  "INFO": [
    {"description": "Snapper is not configured", "fix_available": false}
  ],
  "WARNING": [
    {"description": "The system was not updated in 30 days", "fix_available": true, "id": "stale"}
  ],
  "ERROR": [
    {"description": "The keyring is outdated", "fix_available": true},
    {"description": "No space left on /"}
  ]
}"#;

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
fn every_level_is_read_most_severe_first() {
    assert_eq!(
        parse_health(REPORT),
        Ok(vec![
            issue(HealthLevel::Error, "The keyring is outdated", true),
            issue(HealthLevel::Error, "No space left on /", false),
            issue(
                HealthLevel::Warning,
                "The system was not updated in 30 days",
                true
            ),
            issue(HealthLevel::Info, "Snapper is not configured", false),
        ])
    );
}

#[test]
fn missing_levels_and_empty_output_are_healthy() {
    assert_eq!(parse_health("{}"), Ok(Vec::new()));
    assert_eq!(parse_health(r#"{"WARNING": []}"#), Ok(Vec::new()));
    assert_eq!(parse_health(""), Ok(Vec::new()));
    assert_eq!(parse_health("  \n"), Ok(Vec::new()));
}

#[test]
fn entries_without_description_are_skipped() {
    let report =
        r#"{"ERROR": [{"fix_available": true}, {"description": "  "}, {"description": "real"}]}"#;

    assert_eq!(
        parse_health(report),
        Ok(vec![issue(HealthLevel::Error, "real", false)])
    );
}

#[test]
fn unknown_levels_are_ignored() {
    let report = r#"{"DEBUG": [{"description": "x"}], "INFO": [{"description": "y"}]}"#;

    assert_eq!(
        parse_health(report),
        Ok(vec![issue(HealthLevel::Info, "y", false)])
    );
}

#[test]
fn text_that_is_no_report_fails() {
    for text in [
        "not json",
        "[]",
        r#"{"ERROR": "broken"}"#,
        r#"{"ERROR": [1]}"#,
    ] {
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
    let (_folder, fake) = script(r#"{"ERROR": [{"description": "bad"}]}"#, 1);

    assert_eq!(
        GarudaHealth::with_program(fake).issues(),
        Ok(vec![issue(HealthLevel::Error, "bad", false)])
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
