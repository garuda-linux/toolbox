use toolbox_cli::args::UnitChange;
use toolbox_core::domains::{SystemUnitMasks, SystemUnits, UserUnitMasks, UserUnits};
use toolbox_core::features::services::{ActiveState, does_nothing, plan_control};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_plan::{Plan, UnitName, UnitOperation, UnitScope};

use super::{Refused, fresh};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitLookup {
    Found(ActiveState),
    Missing,
    Unread,
}

/// # Errors
///
/// [`Refused`] for a unit systemd lacks, or a start of a running unit
/// and a stop of a stopped one, as the page greys those out.
pub fn service(
    change: UnitChange,
    scope: UnitScope,
    unit: &UnitName,
    lookup: UnitLookup,
    snapshot: &Snapshot,
) -> Result<Plan, Refused> {
    let name = unit.clone();
    let active = found(unit, lookup)?;

    let operation = match change {
        UnitChange::Start => UnitOperation::Start,
        UnitChange::Stop => UnitOperation::Stop,
        UnitChange::Restart => UnitOperation::Restart,
        UnitChange::Reload => UnitOperation::Reload,
        UnitChange::Enable | UnitChange::Disable | UnitChange::Mask | UnitChange::Unmask => {
            return fresh(|changes| {
                queued(change, scope, name, snapshot, changes);
                Ok(())
            });
        }
    };

    if does_nothing(operation, active) {
        let why = if operation == UnitOperation::Start {
            "runs already"
        } else {
            "is not running"
        };

        return Err(Refused::NothingToDo(format!(
            "Nothing to do: {unit} {why}."
        )));
    }

    Ok(plan_control(operation, scope, name))
}

/// The others run at once and never wait in the queue.
///
/// # Errors
///
/// [`Refused`] for a unit systemd lacks, or a change that runs at once.
#[expect(
    clippy::too_many_arguments,
    reason = "the arguments of `service` and the queue the change joins"
)]
pub fn service_onto(
    change: UnitChange,
    scope: UnitScope,
    unit: &UnitName,
    lookup: UnitLookup,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused> {
    found(unit, lookup)?;

    match change {
        UnitChange::Enable | UnitChange::Disable | UnitChange::Mask | UnitChange::Unmask => {
            queued(change, scope, unit.clone(), snapshot, changes);
            Ok(())
        }
        UnitChange::Start | UnitChange::Stop | UnitChange::Restart | UnitChange::Reload => Err(
            Refused::Rule(format!("{unit} changes at once, never through the queue")),
        ),
    }
}

fn found(unit: &UnitName, lookup: UnitLookup) -> Result<ActiveState, Refused> {
    match lookup {
        UnitLookup::Found(active) => Ok(active),
        UnitLookup::Missing => Err(Refused::Unknown(format!(
            "there is no unit {unit}, see `services list`"
        ))),
        UnitLookup::Unread => Err(Refused::Unavailable(
            "not available: systemd could not be asked for its units".to_owned(),
        )),
    }
}

/// At boot and masks go through the queue, so the plan is the window's.
fn queued(
    change: UnitChange,
    scope: UnitScope,
    unit: UnitName,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) {
    let on = matches!(change, UnitChange::Enable | UnitChange::Mask);

    match (change, scope) {
        (UnitChange::Mask | UnitChange::Unmask, UnitScope::System) => {
            changes.want::<SystemUnitMasks>(unit, Toggle::from_on(on), snapshot);
        }
        (UnitChange::Mask | UnitChange::Unmask, UnitScope::User) => {
            changes.want::<UserUnitMasks>(unit, Toggle::from_on(on), snapshot);
        }
        (_, UnitScope::System) => changes.want::<SystemUnits>(unit, Toggle::from_on(on), snapshot),
        (_, UnitScope::User) => changes.want::<UserUnits>(unit, Toggle::from_on(on), snapshot),
    }
}
