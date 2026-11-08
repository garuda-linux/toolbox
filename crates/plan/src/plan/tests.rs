use super::{Plan, PlannedAction, Segment, StepId};
use crate::{RootAction, UserAction};

fn user_step() -> PlannedAction {
    PlannedAction::User(UserAction::UpgradeAur)
}

/// Each segment's runner and how many steps it holds.
fn shape(plan: Plan) -> Vec<(&'static str, usize)> {
    plan.into_segments()
        .iter()
        .map(|segment| match segment {
            Segment::Root(steps) => ("root", steps.len()),
            Segment::User(steps) => ("user", steps.len()),
        })
        .collect()
}

#[test]
fn steps_are_numbered_from_one_in_order() {
    let plan = Plan::new([
        PlannedAction::Root(RootAction::RefreshKeyring),
        PlannedAction::Root(RootAction::CleanPackageCache),
    ]);

    let ids: Vec<StepId> = plan.steps().iter().map(|step| step.id).collect();
    assert_eq!(ids, [StepId::new(1), StepId::new(2)]);
}

#[test]
fn consecutive_root_steps_share_one_segment() {
    let plan = Plan::new([
        PlannedAction::Root(RootAction::RefreshKeyring),
        PlannedAction::Root(RootAction::CleanPackageCache),
        user_step(),
        PlannedAction::Root(RootAction::RemoveOrphans),
    ]);

    assert_eq!(shape(plan), [("root", 2), ("user", 1), ("root", 1)]);
}

#[test]
fn segments_keep_the_step_ids() {
    let plan = Plan::new([user_step(), PlannedAction::Root(RootAction::RefreshKeyring)]);

    let segments = plan.into_segments();
    let Some(Segment::Root(steps)) = segments.get(1) else {
        panic!("second segment should be root: {segments:?}");
    };

    assert_eq!(steps[0].id, StepId::new(2));
}

#[test]
fn an_empty_plan_has_no_segments() {
    let plan = Plan::new([]);
    assert!(plan.is_empty());
    assert_eq!(plan.into_segments(), Vec::<Segment>::new());
}

#[test]
fn a_step_shows_the_commands_its_runner_executes() {
    let root = PlannedAction::Root(RootAction::CleanPackageCache);
    let user = user_step();

    assert!(root.is_root());
    assert_eq!(root.command_lines(), ["paccache -ruk0"]);
    assert!(!user.is_root());
    assert_eq!(user.command_lines(), ["paru --sudo pkexec -Sua"]);
}

#[test]
fn a_step_of_several_programs_shows_one_line_each() {
    let action = PlannedAction::Root(RootAction::SetHblock { enabled: false });

    assert_eq!(
        action.command_lines(),
        [
            "systemctl disable --now -- hblock.timer",
            "hblock -S none -D none"
        ]
    );
}

#[test]
fn consecutive_user_steps_share_one_segment() {
    let plan = Plan::new([user_step(), user_step()]);

    assert_eq!(shape(plan), [("user", 2)]);
}
