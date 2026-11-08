use toolbox_plan::{PlannedAction, RootAction};

use super::plan_chroot;

#[test]
fn the_chroot_is_one_root_step() {
    let plan = plan_chroot();

    assert_eq!(plan.steps().len(), 1);
    assert_eq!(
        plan.steps()[0].action,
        PlannedAction::Root(RootAction::ChrootInstallation)
    );
}
