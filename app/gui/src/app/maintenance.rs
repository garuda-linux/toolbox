use toolbox_core::features::maintenance::page::{rescue_install, upgrader};
use toolbox_core::features::maintenance::{self, Operation, Program};
use toolbox_plan::{Plan, Toggle};

use super::App;
use crate::command::MaintenanceCommand;
use crate::page::PageId;

impl App {
    pub(super) fn run_maintenance(&mut self, command: &MaintenanceCommand) {
        match *command {
            MaintenanceCommand::RunOperation(operation) => {
                let upgrader =
                    upgrader(&self.current_snapshot(), self.services.home.garuda_update());

                self.run_at_once(maintenance::plan_now(operation.task(upgrader)));
            }
            MaintenanceCommand::RunRepair(repair) => {
                self.run_at_once(repair.plan());
            }
            // Which task is queued depends on garuda-update's program,
            // which Home's service found.
            MaintenanceCommand::Queue { operation, on } => self.queue_operation(operation, on),
            MaintenanceCommand::OpenTool(tool) => {
                let tools = self.services.home.tools();
                let plan =
                    self.services
                        .maintenance
                        .open_tool(tool, tools, self.launcher.as_deref());

                if let Some(plan) = plan {
                    self.run_for_maintenance(plan);
                }
            }
            MaintenanceCommand::InstallGarudaUpdate => {
                if let Some(plan) =
                    rescue_install(Program::of_found(self.services.home.garuda_update()))
                {
                    self.run_at_once(plan);
                }
            }
            // The service takes these, see `hand_to_services`.
            MaintenanceCommand::DismissProblem
            | MaintenanceCommand::Choose { entry: _, on: _ }
            | MaintenanceCommand::Reset => {}
            MaintenanceCommand::Reveal(tab) => {
                self.pages.show_maintenance_tab(tab);
                self.state.page = PageId::Maintenance;
            }
            // The programs reader is Home's, which Maintenance's service cannot reach.
            MaintenanceCommand::Refresh => {
                self.services.home.refresh_programs();
                self.read_system();
            }
        }
    }

    /// A plan the page waits for: one the Console refused never started.
    pub(super) fn run_for_maintenance(&mut self, plan: Plan) {
        if !self.run_at_once(plan) {
            self.services.maintenance.plan_refused();
        }
    }

    fn queue_operation(&mut self, operation: Operation, on: bool) {
        let snapshot = self.current_snapshot();
        let task = operation.task(upgrader(&snapshot, self.services.home.garuda_update()));

        self.queue.edit(&snapshot, |changes, snapshot| {
            maintenance::queue(task, Toggle::from_on(on), changes, snapshot);
        });
    }
}
