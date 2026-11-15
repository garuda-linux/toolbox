//! The Diagnostics page on the real system: the full report collected,
//! which always holds its headings, whatever this machine lacks, and read
//! with its lines wrapped. Then its upload is asked for, turned down once
//! and sent to the dry-run paste server, which keeps nothing.

use super::step::Step;

#[must_use = "the steps do nothing until a suite runs them"]
pub fn steps() -> Vec<Step> {
    vec![
        Step::WaitGone("Reading the system…"),
        Step::tab("Diagnostics"),
        Step::wait("Reports"),
        Step::Shot("diagnostics"),
        Step::button("Collect Full report"),
        Step::wait("Copy report"),
        Step::Shot("diagnostics-report"),
        // Long lines wrap at the viewer's edge and scroll sideways again.
        Step::click("Wrap lines"),
        Step::Shot("diagnostics-wrapped"),
        Step::click("Wrap lines"),
        Step::click("Share report…"),
        Step::wait("Upload this report?"),
        Step::click("Cancel"),
        Step::WaitGone("Upload this report?"),
        Step::click("Share report…"),
        Step::wait("Upload this report?"),
        Step::Shot("diagnostics-question"),
        Step::click("Upload"),
        Step::wait("Copy link"),
        Step::Shot("diagnostics-uploaded"),
    ]
}
