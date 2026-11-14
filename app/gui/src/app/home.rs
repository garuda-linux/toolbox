//! What the app does with a click on Home that its service cannot: a run
//! at once, a restart, the installer and the end of the Setup Assistant.

use std::path::Path;

use toolbox_core::features::live::{INSTALLER, INSTALLER_ARGS};
use toolbox_core::features::maintenance::{Program, RunNow};
use toolbox_core::features::recent::remember;

use super::App;
use crate::command::HomeCommand;
use crate::page::PageId;
use crate::pages::home::HomeProblem;

impl App {
    pub(super) fn run_home(&mut self, command: HomeCommand) {
        match command {
            HomeCommand::RunNow(run) => self.run_now(run),
            // The page disables Restart while a plan runs. A command from
            // elsewhere waits for the plan the same way.
            HomeCommand::Restart if self.console.running() => {}
            HomeCommand::Restart => self.services.home.restart(),
            HomeCommand::StartInstaller => self.start_installer(),
            HomeCommand::SetupFinished => self.state.setup_finished = true,
            // The service takes these, see `hand_to_services`.
            HomeCommand::QueueUpgrade(_) | HomeCommand::DismissProblem | HomeCommand::Refresh => {}
        }
    }

    fn run_now(&mut self, run: RunNow) {
        let plan = run.plan(
            &self.current_snapshot(),
            Program::of_found(self.services.home.garuda_update()),
        );

        self.run_at_once(plan);
    }

    /// The installer asks for root itself. The launcher is shared with
    /// Maintenance and Network, so App holds it.
    fn start_installer(&mut self) {
        let Some(launcher) = &self.launcher else {
            return;
        };

        let problem = launcher
            .start(Path::new(INSTALLER), &INSTALLER_ARGS)
            .err()
            .map(|error| {
                tracing::error!("cannot start the installer: {error}");
                HomeProblem::Installer(error.to_string())
            });

        self.services.home.set_problem(problem);
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
