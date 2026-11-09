//! Saved changes that did not come back at start.

use egui_kittest::kittest::Queryable as _;

use super::page;
use crate::command::{Command, HomeCommand};
use crate::pages::home::HomeProblem;
use crate::testing::PageFixture;

#[test]
fn lost_changes_are_named_until_dismissed() {
    let mut harness = page(PageFixture {
        problem: Some(HomeProblem::QueueLost(3)),
        ..PageFixture::default()
    });

    assert!(
        harness
            .query_by_label_contains("3 queued changes could not be restored")
            .is_some()
    );

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(HomeCommand::DismissProblem)]
    );
}

#[test]
fn a_whole_queue_says_nothing() {
    let harness = page(PageFixture::default());

    assert!(
        harness
            .query_by_label_contains("could not be restored")
            .is_none()
    );
}
