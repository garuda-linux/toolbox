//! What the app does with a click on the Services page: control a unit at
//! once and show its journal. Its service takes the other clicks.

use toolbox_core::features::services::{plan_control, plan_logs};

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
            // The service takes these, see `hand_to_services`.
            ServiceCommand::Enable {
                scope: _,
                unit: _,
                on: _,
            }
            | ServiceCommand::Mask {
                scope: _,
                unit: _,
                on: _,
            }
            | ServiceCommand::Refresh(_) => {}
        }
    }
}
