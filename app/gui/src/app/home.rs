//! What the app does with a click on Home.

use std::path::Path;

use toolbox_core::features::live::{INSTALLER, INSTALLER_ARGS};
use toolbox_core::features::maintenance::{Program, RunNow, queue_upgrade};
use toolbox_core::features::recent::remember;
use toolbox_plan::Toggle;

use super::App;
use crate::command::HomeCommand;
use crate::page::PageId;
use crate::pages::home::HomeProblem;

impl App {
    pub(super) fn run_home(&mut self, command: HomeCommand) {
        match command {
            HomeCommand::RunNow(run) => self.run_now(run),
            HomeCommand::QueueUpgrade(on) => {
                let snapshot = self.current_snapshot();
                let garuda_update = self.garuda_update();
                self.queue.edit(&snapshot, |changes, snapshot| {
                    queue_upgrade(
                        Toggle::from_on(on),
                        changes,
                        snapshot,
                        Program::of_found(garuda_update),
                    );
                });
            }
            // The page disables Restart while a plan runs. A command from
            // elsewhere waits for the plan the same way.
            HomeCommand::Restart if self.console.running() => {}
            HomeCommand::Restart => {
                self.home_problem = None;
                if let Some(power) = &self.power {
                    power.restart(self.restart_refused.0.clone());
                }
            }
            HomeCommand::StartInstaller => self.start_installer(),
            HomeCommand::SetupFinished => self.state.setup_finished = true,
            HomeCommand::DismissProblem => self.home_problem = None,
            HomeCommand::Refresh => {
                self.status.refresh();
                self.facts.refresh();
                self.news.refresh();
                self.programs.refresh();
                self.read_system();
            }
        }
    }

    fn run_now(&mut self, run: RunNow) {
        let plan = run.plan(
            &self.current_snapshot(),
            Program::of_found(self.garuda_update()),
        );

        self.run_at_once(plan);
    }

    /// The installer asks for root itself.
    fn start_installer(&mut self) {
        let Some(launcher) = &self.launcher else {
            return;
        };

        self.home_problem = launcher
            .start(Path::new(INSTALLER), &INSTALLER_ARGS)
            .err()
            .map(|error| {
                tracing::error!("cannot start the installer: {error}");
                HomeProblem::Installer(error.to_string())
            });
    }

    /// Whether the last check found garuda-update's program, for a system
    /// whose installed packages are not known.
    pub(super) fn garuda_update(&self) -> bool {
        self.programs
            .latest()
            .is_some_and(|programs| programs.garuda_update)
    }
}

impl App {
    /// Notes the open page for Home's shortcuts. Home itself is no
    /// shortcut on Home.
    pub(super) fn remember_page(&mut self) {
        let page = self.state.page;
        if page != PageId::Home && self.state.recent.first() != Some(&page) {
            remember(&mut self.state.recent, page);
        }
    }
}
