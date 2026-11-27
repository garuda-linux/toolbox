use strum::IntoEnumIterator as _;
use toolbox_plan::{PlannedAction, RootAction, TargetList, UserAction};

use super::{HelperFailure, asks_review};
use crate::test_support::package;

#[test]
fn every_failure_has_its_stable_code() {
    let codes: Vec<&str> = HelperFailure::iter().map(HelperFailure::as_str).collect();

    assert_eq!(
        codes,
        [
            "missing-secret",
            "start",
            "pkexec",
            "no-pipes",
            "lost",
            "ended",
            "user-steps"
        ]
    );
}

#[test]
fn only_the_review_of_the_aur_build_asks_for_an_answer() {
    let review = PlannedAction::User(UserAction::ReviewAur {
        bases: TargetList::new([package("octopi-git")]).unwrap(),
    });

    let refresh = PlannedAction::Root(RootAction::RefreshKeyring);

    assert!(asks_review(&review.commands().join(" ")));
    assert!(!asks_review(&refresh.commands().join(" ")));
}
