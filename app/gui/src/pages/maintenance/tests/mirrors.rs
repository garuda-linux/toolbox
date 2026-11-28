//! What the page says when the mirror ranking did not start.

use egui_kittest::kittest::Queryable as _;
use toolbox_system::mirrors::{RankProblem, StartProblem};

use super::{page, programs, with};
use crate::maintenance::{Maintenance, MaintenanceProblem};

#[test]
fn a_failed_ranking_names_the_system_reason() {
    let problem = MaintenanceProblem::Mirrors(RankProblem::NotStarted(StartProblem::Failed(
        "denied".to_owned(),
    )));

    let maintenance = Maintenance::showing(Some(problem), false, Vec::new(), None);

    let harness = page(with(programs(), maintenance));

    assert!(
        harness
            .query_by_label_contains("rate-mirrors could not be started")
            .is_some()
    );
    assert!(harness.query_by_label_contains(": denied").is_some());
}

/// The program's absence is a word of the window's, not the system's.
#[test]
fn a_missing_ranking_program_is_named_in_words() {
    let problem = MaintenanceProblem::Mirrors(RankProblem::NotStarted(StartProblem::Missing));
    let maintenance = Maintenance::showing(Some(problem), false, Vec::new(), None);
    let harness = page(with(programs(), maintenance));

    assert!(
        harness
            .query_by_label_contains(": the program is not installed")
            .is_some()
    );
}
