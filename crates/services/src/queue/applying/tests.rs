use toolbox_plan::{Plan, PlannedAction, RootAction, StepId};

use super::Applying;

fn plan() -> Plan {
    Plan::new([
        PlannedAction::Root(RootAction::RefreshKeyring),
        PlannedAction::Root(RootAction::RemoveOrphans),
    ])
}

#[test]
fn the_end_of_the_queues_plan_names_the_actions_that_succeeded() {
    let mut applying = Applying::Sent(plan()).started();

    let done = applying.finished(|step| step == StepId::new(1));

    assert_eq!(
        done,
        Some(vec![PlannedAction::Root(RootAction::RefreshKeyring)])
    );
    assert_eq!(applying, Applying::None);
}

/// Another plan ran when Apply sent the queue's, so its end is not the
/// queue's end.
#[test]
fn the_end_of_another_plan_settles_nothing() {
    let mut applying = Applying::Sent(plan());

    assert_eq!(applying.finished(|_| true), None);
    assert_eq!(applying, Applying::Sent(plan()));
}

#[test]
fn a_refused_plan_is_forgotten() {
    assert_eq!(Applying::Sent(plan()).refused(), Applying::None);
    assert_eq!(
        Applying::Running(plan()).refused(),
        Applying::Running(plan())
    );
}
