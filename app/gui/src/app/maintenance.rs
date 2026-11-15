use toolbox_core::features::maintenance::page::{rescue_install, upgrader};
use toolbox_core::features::maintenance::{self, Operation};
use toolbox_plan::{Plan, Toggle};

use super::App;
use crate::command::MaintenanceCommand;
use crate::page::PageId;
use crate::page_service::Owner;

impl App {
    pub(super) fn run_maintenance(&mut self, ctx: &egui::Context, command: &MaintenanceCommand) {
        match *command {
            MaintenanceCommand::RunOperation(operation) => {
                let upgrader =
                    upgrader(&self.current_snapshot(), self.services.home.garuda_update());

                self.run_on_maintenance(maintenance::plan_now(operation.task(upgrader)));
            }
            MaintenanceCommand::RunRepair(repair) => self.run_on_maintenance(repair.plan()),
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
            MaintenanceCommand::InstallMissing => {
                if let Some(plan) = rescue_install(self.services.home.repairs()) {
                    self.run_on_maintenance(plan);
                }
            }
            // The service takes these, see `hand_to_services`.
            MaintenanceCommand::DismissProblem
            | MaintenanceCommand::Choose { entry: _, on: _ }
            | MaintenanceCommand::Reset => {}
            MaintenanceCommand::Reveal(tab) => {
                self.pages.show_maintenance_tab(tab);
                self.navigate(ctx, PageId::Maintenance);
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
        if !self.run_owned(plan, Owner::Maintenance) {
            self.services.maintenance.plan_refused();
            self.services.maintenance.refuse_busy();
        }
    }

    /// A run the page does not wait for. The page says when the Console
    /// refused it.
    fn run_on_maintenance(&mut self, plan: Plan) {
        if self.run_at_once(plan).is_none() {
            self.services.maintenance.refuse_busy();
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
