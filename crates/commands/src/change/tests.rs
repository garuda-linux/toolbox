use toolbox_cli::Exit;
use toolbox_cli::args::UnitChange;
use toolbox_core::Snapshot;
use toolbox_core::features::services::ActiveState;
use toolbox_plan::{PlannedAction, RootAction, UnitName, UnitOperation, UnitScope, Upgrader};

use super::{Refused, UnitLookup, service, task};

fn unit() -> UnitName {
    UnitName::new("sshd.service").unwrap()
}

#[test]
fn starting_a_running_service_is_nothing_to_do_and_ends_with_0() {
    let refused = service(
        UnitChange::Start,
        UnitScope::System,
        &unit(),
        UnitLookup::Found(ActiveState::Active),
        &Snapshot::default(),
    )
    .unwrap_err();

    assert_eq!(refused.exit(), Exit::Success);
    assert!(refused.message().contains("runs already"));
}

#[test]
fn stopping_a_stopped_service_is_nothing_to_do() {
    let refused = service(
        UnitChange::Stop,
        UnitScope::User,
        &unit(),
        UnitLookup::Found(ActiveState::Inactive),
        &Snapshot::default(),
    )
    .unwrap_err();

    assert_eq!(refused.exit(), Exit::Success);
}

#[test]
fn a_restart_runs_one_control_step() {
    let plan = service(
        UnitChange::Restart,
        UnitScope::System,
        &unit(),
        UnitLookup::Found(ActiveState::Active),
        &Snapshot::default(),
    )
    .unwrap();

    assert_eq!(plan.steps().len(), 1);
    assert!(matches!(
        plan.steps()[0].action,
        PlannedAction::Root(RootAction::ControlUnits {
            operation: UnitOperation::Restart,
            units: _,
        })
    ));
}

#[test]
fn enable_and_mask_plan_a_change() {
    for change in [UnitChange::Enable, UnitChange::Mask] {
        let plan = service(
            change,
            UnitScope::System,
            &unit(),
            UnitLookup::Found(ActiveState::Inactive),
            &Snapshot::default(),
        )
        .unwrap();

        assert_ne!(plan.steps().len(), 0, "{change:?} planned nothing");
    }
}

#[test]
fn an_unknown_task_is_not_available() {
    assert!(matches!(
        task("no-such-task", Upgrader::Pacman),
        Err(Refused::Unknown(_))
    ));
}

#[test]
fn a_destructive_repair_wants_its_name_typed() {
    let (plan, typed) = task("remote-full-fix", Upgrader::GarudaUpdate).unwrap();
    assert_ne!(plan.steps().len(), 0);
    assert_eq!(typed.as_deref(), Some("remote-full-fix"));

    let (_, cache) = task("clean-package-cache", Upgrader::GarudaUpdate).unwrap();
    assert_eq!(cache.as_deref(), Some("clean-package-cache"));

    let (_, keys) = task("refresh-keyring", Upgrader::GarudaUpdate).unwrap();
    assert_eq!(keys, None);
}

#[test]
fn a_unit_the_system_does_not_have_is_not_available() {
    for change in [UnitChange::Start, UnitChange::Enable, UnitChange::Mask] {
        let refused = service(
            change,
            UnitScope::System,
            &UnitName::new("sshdd.service").unwrap(),
            UnitLookup::Missing,
            &Snapshot::default(),
        )
        .unwrap_err();

        assert_eq!(refused.exit(), Exit::Unavailable, "{change:?}");
        assert!(refused.message().contains("services list"));
    }
}

#[test]
fn units_that_could_not_be_read_plan_nothing() {
    let refused = service(
        UnitChange::Restart,
        UnitScope::System,
        &unit(),
        UnitLookup::Unread,
        &Snapshot::default(),
    )
    .unwrap_err();

    assert_eq!(refused.exit(), Exit::Unavailable);
}

#[test]
fn nothing_to_do_is_a_plain_line_and_a_refusal_an_error() {
    let paint = toolbox_cli::output::Paint::new(toolbox_cli::output::Style::Plain);

    let nothing = Refused::NothingToDo("Nothing to do: sshd.service runs already.".to_owned());
    assert_eq!(
        nothing.line(paint),
        "Nothing to do: sshd.service runs already."
    );

    let unknown = Refused::Unknown("no task is called `x`".to_owned());
    assert_eq!(unknown.line(paint), "error: no task is called `x`");
}

#[test]
fn an_unknown_name_hints_at_the_closest_known_one() {
    let paint = toolbox_cli::output::Paint::new(toolbox_cli::output::Style::Plain);

    let refused = task("clean-package-cach", Upgrader::Pacman).unwrap_err();
    assert_eq!(
        refused.line(paint),
        "error: no task is called `clean-package-cach`, see `maintenance list`\n\
         hint: did you mean `clean-package-cache`?"
    );
    assert_eq!(refused.exit(), Exit::Unavailable);

    let far = task("x", Upgrader::Pacman).unwrap_err();
    assert!(!far.line(paint).contains("hint:"));
}

#[test]
fn a_missing_service_suffix_is_hinted() {
    let refused = Refused::Unknown("there is no unit sshd".to_owned())
        .hinted("sshd", ["bluetooth.service", "sshd.service"]);

    assert!(refused.message().ends_with("did you mean `sshd.service`?"));
}
