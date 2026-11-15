use super::{Owner, Waiting};
use crate::apply::PlanId;

#[test]
fn each_answer_reaches_the_sender_of_its_plan_once() {
    let mut waiting = Waiting::default();
    waiting.sent(PlanId::new(1), Owner::Maintenance);
    waiting.sent(PlanId::new(2), Owner::BootRepair);

    assert_eq!(waiting.answered(PlanId::new(2)), Some(Owner::BootRepair));
    assert_eq!(waiting.answered(PlanId::new(1)), Some(Owner::Maintenance));
    assert_eq!(waiting.answered(PlanId::new(1)), None, "answered already");
}

#[test]
fn a_plan_nobody_waits_for_has_no_owner() {
    let mut waiting = Waiting::default();
    waiting.sent(PlanId::new(1), Owner::Users);

    assert_eq!(waiting.answered(PlanId::new(3)), None);
    assert_eq!(waiting.answered(PlanId::new(1)), Some(Owner::Users));
}
