//! The garuda-health row of Home: CRITICAL and HIGH are problems, LOW a
//! warning, INFO no row at all.

use egui_kittest::kittest::Queryable as _;
use toolbox_system::{HealthIssue, HealthLevel};

use crate::home::{home, host, quiet};

const PROBLEMS: &str = "garuda-health found problems";
const WARNINGS: &str = "garuda-health found warnings";

/// Whether Home shows [`PROBLEMS`] and [`WARNINGS`] for one issue of
/// `level`. The message is garuda-health 1.11.1's own.
fn shown_for(level: HealthLevel) -> (bool, bool) {
    let mut status = quiet();
    status.health = Ok(vec![HealthIssue {
        level,
        message: "Important garuda linux packages are missing".to_owned(),
        fix_available: true,
    }]);
    let (harness, _runtime, _power) = home(host(false, false), status);

    (
        harness.query_by_label(PROBLEMS).is_some(),
        harness.query_by_label(WARNINGS).is_some(),
    )
}

#[test]
fn critical_and_high_are_problems() {
    assert_eq!(shown_for(HealthLevel::Critical), (true, false));
    assert_eq!(shown_for(HealthLevel::High), (true, false));
}

#[test]
fn low_is_a_warning_and_info_shows_nothing() {
    assert_eq!(shown_for(HealthLevel::Low), (false, true));
    assert_eq!(shown_for(HealthLevel::Info), (false, false));
}
