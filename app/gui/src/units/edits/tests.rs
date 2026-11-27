use toolbox_core::{ChangeSet, Snapshot};
use toolbox_plan::{UnitName, UnitOperation, UnitScope};

use crate::command::{AppCommand, ServiceCommand};
use crate::units::Units;

fn sshd() -> UnitName {
    UnitName::new("sshd.service").unwrap()
}

#[test]
fn a_boot_wish_is_queued() {
    let command = AppCommand::Service(ServiceCommand::Enable {
        scope: UnitScope::System,
        unit: sshd(),
        on: toolbox_core::Toggle::On,
    });

    let handled = Units::default().handle(&command, &ChangeSet::default(), &Snapshot::default());

    assert!(handled.taken);
    assert!(handled.edit.is_some_and(|edit| edit.len() == 1));
}

#[test]
fn a_mask_is_queued() {
    let command = AppCommand::Service(ServiceCommand::Mask {
        scope: UnitScope::User,
        unit: sshd(),
        on: toolbox_core::Toggle::On,
    });

    let handled = Units::default().handle(&command, &ChangeSet::default(), &Snapshot::default());

    assert!(handled.taken);
    assert!(handled.edit.is_some_and(|edit| edit.len() == 1));
}

/// A control runs at once in App's Console.
#[test]
fn a_control_is_left_to_app() {
    let command = AppCommand::Service(ServiceCommand::Control {
        scope: UnitScope::System,
        unit: sshd(),
        operation: UnitOperation::Restart,
    });

    let handled = Units::default().handle(&command, &ChangeSet::default(), &Snapshot::default());

    assert!(!handled.taken);
}

/// Without the task nothing is listed, but the click is the service's.
#[test]
fn a_refresh_is_the_services() {
    let command = AppCommand::Service(ServiceCommand::Refresh(UnitScope::User));

    let handled = Units::default().handle(&command, &ChangeSet::default(), &Snapshot::default());

    assert!(handled.taken);
    assert!(handled.edit.is_none());
}
