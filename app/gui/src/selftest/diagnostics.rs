//! The Diagnostics page on the real system: the full report collected,
//! which always holds its headings, whatever this machine lacks. Then its
//! upload is asked for, turned down once and sent to the dry-run paste
//! server, which keeps nothing.

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Diagnostics"),
        Step::wait("Choose a report and click Collect."),
        Step::Shot("diagnostics"),
        Step::combo("Report to collect"),
        Step::click("Full report"),
        Step::click("Collect"),
        Step::wait("Copy report"),
        Step::Shot("diagnostics-report"),
        Step::click("Share report…"),
        Step::wait("Upload this report?"),
        Step::click("Cancel"),
        Step::WaitGone("Upload this report?"),
        Step::click("Share report…"),
        Step::wait("Upload this report?"),
        Step::Shot("diagnostics-question"),
        Step::click("Upload"),
        Step::wait("Link to the uploaded report"),
        Step::Shot("diagnostics-uploaded"),
    ]
}
