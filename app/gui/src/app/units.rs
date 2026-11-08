//! What the app does with a click on the Services page: control a unit at
//! once, show its journal, queue its boot and mask state, list again.

use toolbox_core::Toggle;
use toolbox_core::domains::{SystemUnitMasks, SystemUnits, UserUnitMasks, UserUnits};
use toolbox_core::features::services::{plan_control, plan_logs};
use toolbox_plan::UnitScope;

use super::App;
use crate::command::ServiceCommand;

impl App {
    pub(super) fn run_service(&mut self, command: ServiceCommand) {
        match command {
            ServiceCommand::Control {
                scope,
                unit,
                operation,
            } => {
                self.run_at_once(plan_control(operation, scope, unit));
            }
            ServiceCommand::Logs { scope, unit } => {
                self.run_at_once(plan_logs(scope, unit));
            }
            ServiceCommand::Enable { scope, unit, on } => {
                let snapshot = self.current_snapshot();
                let wish = Toggle::from_on(on);
                self.queue.edit(&snapshot, |changes, snapshot| match scope {
                    UnitScope::System => changes.want::<SystemUnits>(unit, wish, snapshot),
                    UnitScope::User => changes.want::<UserUnits>(unit, wish, snapshot),
                });
            }
            ServiceCommand::Mask { scope, unit, on } => {
                let snapshot = self.current_snapshot();
                let wish = Toggle::from_on(on);
                self.queue.edit(&snapshot, |changes, snapshot| match scope {
                    UnitScope::System => changes.want::<SystemUnitMasks>(unit, wish, snapshot),
                    UnitScope::User => changes.want::<UserUnitMasks>(unit, wish, snapshot),
                });
            }
            ServiceCommand::Refresh(scope) => {
                if let Some(units) = &self.units {
                    units.refresh(scope);
                }
            }
        }
    }
}
