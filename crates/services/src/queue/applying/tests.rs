use toolbox_core::Owners;
use toolbox_plan::{Plan, PlannedAction, RootAction};

use super::Applying;

fn plan() -> Plan {
    Plan::new([
        PlannedAction::Root(RootAction::RefreshKeyring),
        PlannedAction::Root(RootAction::RemoveOrphans),
    ])
}

fn sent() -> Applying {
    Applying::Sent(plan(), Owners::default())
}

#[test]
fn the_end_of_the_queues_plan_hands_back_the_plan() {
    let mut applying = sent().started();

    assert_eq!(applying.finished(), Some((plan(), Owners::default())));
    assert_eq!(applying, Applying::None);
}

/// Another plan ran when Apply sent the queue's, so its end is not the
/// queue's end.
#[test]
fn the_end_of_another_plan_settles_nothing() {
    let mut applying = sent();

    assert_eq!(applying.finished(), None);
    assert_eq!(applying, sent());
}

#[test]
fn a_refused_plan_is_forgotten() {
    assert_eq!(sent().refused(), Applying::None);
    assert_eq!(sent().started().refused(), sent().started());
}
