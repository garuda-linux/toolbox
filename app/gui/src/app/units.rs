//! What the app does with a click on the Services page: control a unit at
//! once, show its journal, queue its boot and mask state, list again.

use toolbox_core::Toggle;
use toolbox_core::features::services::{self, UnitWish, plan_control, plan_logs};
use toolbox_plan::{UnitName, UnitScope};

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
                self.queue_unit(scope, unit, UnitWish::AtBoot(Toggle::from_on(on)));
            }
            ServiceCommand::Mask { scope, unit, on } => {
                self.queue_unit(scope, unit, UnitWish::Masked(Toggle::from_on(on)));
            }
            ServiceCommand::Refresh(scope) => {
                if let Some(units) = &self.units {
                    units.refresh(scope);
                }
            }
        }
    }

    fn queue_unit(&mut self, scope: UnitScope, unit: UnitName, wish: UnitWish) {
        let snapshot = self.current_snapshot();
        self.queue.edit(&snapshot, |changes, snapshot| {
            services::queue(changes, scope, unit, wish, snapshot);
        });
    }
}
