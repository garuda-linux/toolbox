use std::path::Path;

use toolbox_core::features::maintenance::page::{rescue_install, run_now_plan, task, upgrader};
use toolbox_core::features::maintenance::{Operations, Program};
use toolbox_plan::{Plan, Toggle};

use super::App;
use crate::command::MaintenanceCommand;
use crate::page::PageId;

impl App {
    pub(super) fn run_maintenance(&mut self, command: MaintenanceCommand) {
        match command {
            MaintenanceCommand::RunNow(id) => {
                let upgrader = upgrader(&self.current_snapshot(), self.garuda_update());
                if let Some(plan) = run_now_plan(&id, upgrader) {
                    self.run_at_once(plan);
                }
            }
            MaintenanceCommand::Queue { operation, on } => self.queue_operation(&operation, on),
            MaintenanceCommand::OpenTool(id) => {
                let tools = self
                    .programs
                    .latest()
                    .map_or(&[][..], |programs| programs.tools.as_slice());

                let plan = self
                    .maintenance
                    .open_tool(&id, tools, self.launcher.as_deref());

                if let Some(plan) = plan {
                    self.run_for_maintenance(plan);
                }
            }
            MaintenanceCommand::InstallGarudaUpdate => {
                if let Some(plan) = rescue_install(Program::of_found(self.garuda_update())) {
                    self.run_at_once(plan);
                }
            }
            MaintenanceCommand::DismissProblem => self.maintenance.dismiss_problem(),
            MaintenanceCommand::Choose { entry, on } => {
                self.maintenance.choose(&entry, Toggle::from_on(on));
            }
            MaintenanceCommand::Reset => self.maintenance.reset(self.catalogs.skel.as_ref()),
            MaintenanceCommand::Reveal(tab) => {
                self.pages.show_maintenance_tab(tab);
                self.state.page = PageId::Maintenance;
            }
            MaintenanceCommand::Refresh => {
                self.programs.refresh();
                self.read_system();
            }
        }
    }

    /// A plan the page waits for: one the Console refused never started.
    pub(super) fn run_for_maintenance(&mut self, plan: Plan) {
        if !self.run_at_once(plan) {
            self.maintenance.plan_refused();
        }
    }

    /// A ranked mirrorlist runs at once, which opens the Console.
    pub(super) fn take_maintenance(&mut self) {
        if let Some(plan) = self.maintenance.take() {
            self.run_at_once(plan);
        }
    }

    /// The last check of the programs may predate the end, so the disk is asked.
    pub(super) fn maintenance_finished(&mut self, succeeded: bool) {
        self.maintenance
            .plan_finished(succeeded, Path::exists, self.launcher.as_deref());
    }

    fn queue_operation(&mut self, operation: &str, on: bool) {
        let snapshot = self.current_snapshot();
        let Some(task) = task(operation, upgrader(&snapshot, self.garuda_update())) else {
            return;
        };

        self.queue.edit(&snapshot, |changes, snapshot| {
            if on {
                Operations::want(changes, task, snapshot);
            } else {
                changes.remove::<Operations>(&task.operation());
            }
        });
    }
}
