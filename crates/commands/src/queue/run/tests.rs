use toolbox_cli::Exit;
use toolbox_plan::StepId;

use super::settles;
use crate::apply::Ended;

fn ended(exit: Exit, succeeded: &[u32]) -> Ended {
    Ended {
        exit,
        asked_not: false,
        succeeded: succeeded.iter().copied().map(StepId::new).collect(),
        skipped: Vec::new(),
    }
}

/// A no at the question before the plan ran nothing, so the queue stays as
/// it is.
#[test]
fn a_declined_plan_that_ran_nothing_settles_nothing() {
    assert!(!settles(false, &ended(Exit::Declined, &[])));
    assert!(!settles(false, &ended(Exit::NeedsYes, &[])));
}

/// A no at the review of an AUR build also ends as declined, after the
/// steps before it ran. Those are settled, or a one-off task would run
/// again with the next apply.
#[test]
fn a_declined_review_settles_the_steps_that_ran_before_it() {
    assert!(settles(false, &ended(Exit::Declined, &[1])));
}

#[test]
fn a_dry_run_settles_nothing() {
    assert!(!settles(true, &ended(Exit::Success, &[1])));
}

#[test]
fn a_run_that_failed_or_was_interrupted_is_settled() {
    assert!(settles(false, &ended(Exit::Failure, &[])));
    assert!(settles(false, &ended(Exit::Interrupted, &[1])));
}
