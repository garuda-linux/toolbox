use std::path::PathBuf;

use serde_json::{Map, Value};
use tempfile::TempDir;
use toolbox_text::plain;

use super::{GarudaHealth, HealthIssue, HealthLevel, HealthSource as _, parse_health};
use crate::ProbeError;
use crate::test_support::fake_program;

/// `garuda-health --json` 1.11.1 in the Lima VM, which exited with 1.
const REPORT: &str = include_str!("fixtures/garuda-health.json");

/// The first LOW description of the capture. The edge cases below change
/// the capture around it and name each change in their doc comment.
const FIRST_LOW: &str =
    "Failed systemd services found: cloud-init-main.service, toolbox-fixture-failed.service";

fn issue(level: HealthLevel, message: &str, fix_available: bool) -> HealthIssue {
    HealthIssue {
        level,
        message: plain(message),
        fix_available,
    }
}

fn real_issues() -> Vec<HealthIssue> {
    parse_health(REPORT).unwrap()
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

/// The capture with its INFO key renamed to CRITICAL, so the CRITICAL list
/// comes after the LOW list in the file.
#[test]
fn critical_comes_before_low() {
    let report = REPORT.replacen(r#""INFO": ["#, r#""CRITICAL": ["#, 1);

    let levels: Vec<HealthLevel> = parse_health(&report)
        .unwrap()
        .into_iter()
        .map(|issue| issue.level)
        .collect();

    assert_eq!(
        levels,
        [
            HealthLevel::Critical,
            HealthLevel::Critical,
            HealthLevel::Critical,
            HealthLevel::Critical,
            HealthLevel::Low,
            HealthLevel::Low,
        ]
    );
    assert!(HealthLevel::Critical > HealthLevel::High);
    assert!(HealthLevel::High > HealthLevel::Low);
    assert!(HealthLevel::Low > HealthLevel::Info);
}

/// The capture with every list emptied, and then with no level at all:
/// both are a healthy system. Empty output is one too.
#[test]
fn emptied_lists_and_empty_output_are_healthy() {
    let mut report: Map<String, Value> = serde_json::from_str(REPORT).unwrap();

    for list in report.values_mut() {
        *list = Value::Array(Vec::new());
    }

    assert_eq!(
        parse_health(&Value::Object(report).to_string()),
        Ok(Vec::new())
    );
    assert_eq!(
        parse_health(&Value::Object(Map::new()).to_string()),
        Ok(Vec::new())
    );
    assert_eq!(parse_health(""), Ok(Vec::new()));
    assert_eq!(parse_health("  \n"), Ok(Vec::new()));
}

/// The capture with the first LOW description blanked, then with its
/// `description` key removed. Both entries are left out, the others stay.
#[test]
fn entries_without_description_are_skipped() {
    let blank = REPORT.replacen(FIRST_LOW, "  ", 1);
    let missing = REPORT.replacen(&format!("\"description\": \"{FIRST_LOW}\","), "", 1);
    let rest: Vec<HealthIssue> = real_issues().into_iter().skip(1).collect();

    assert_eq!(parse_health(&blank), Ok(rest.clone()));
    assert_eq!(parse_health(&missing), Ok(rest));
}

/// The capture with its LOW key renamed to ERROR, a level garuda-health
/// does not print. Its entries are left out and the INFO ones stay.
#[test]
fn unknown_levels_are_ignored() {
    let report = REPORT.replacen(r#""LOW": ["#, r#""ERROR": ["#, 1);
    let info: Vec<HealthIssue> = real_issues().into_iter().skip(2).collect();

    assert_eq!(parse_health(&report), Ok(info));
}

/// The capture cut in half, wrapped in an array, with its LOW value turned
/// into a string (its list kept under an added `FIXTURE` key so the JSON
/// stays whole), and with a number as its first LOW entry. None of these
/// is a report.
#[test]
fn text_that_is_no_report_fails() {
    let cut = REPORT.get(..REPORT.len() / 2).unwrap_or_default();
    let array = format!("[{REPORT}]");
    let level_not_list = REPORT.replacen(r#""LOW": ["#, r#""LOW": "broken", "FIXTURE": ["#, 1);
    let number_entry = REPORT.replacen(r#""LOW": ["#, r#""LOW": [1, "#, 1);

    for text in [
        cut,
        array.as_str(),
        level_not_list.as_str(),
        number_entry.as_str(),
    ] {
        assert!(
            matches!(parse_health(text), Err(error) if error.is_failure()),
            "{text}"
        );
    }
}

/// The capture printed with the exit code it had, 1.
#[test]
fn a_report_counts_whatever_the_exit_code() {
    let (_folder, fake) = script(REPORT, 1);

    assert_eq!(GarudaHealth::with_program(fake).issues(), Ok(real_issues()));
}

#[test]
fn a_failure_without_report_fails() {
    let (_folder, fake) = script("", 1);
    let result = GarudaHealth::with_program(fake).issues();

    assert!(
        matches!(&result, Err(ProbeError::Command { command_line: _, status: _, stderr }) if stderr.contains("health: complaint")),
        "{result:?}"
    );
}

#[test]
fn a_missing_garuda_health_is_unavailable() {
    let health = GarudaHealth::with_program("toolbox-system-no-such-program");

    assert_eq!(health.issues(), Err(ProbeError::Unavailable));
}
