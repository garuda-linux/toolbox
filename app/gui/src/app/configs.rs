//! What the app does for the Config Files page: it shows the Console, and
//! runs the editor and the write in it. A cancel stops the editor.

use super::App;
use crate::apply::ApplyCommand;
use crate::command::ConfigsCommand;
use crate::configs::Act;
use crate::page_service::Owner;

impl App {
    pub(super) const fn run_configs(&mut self, command: &ConfigsCommand) {
        match command {
            ConfigsCommand::ShowConsole => self.state.console_open = true,
            // The service takes these, see `hand_to_services`.
            ConfigsCommand::Refresh
            | ConfigsCommand::Open(_)
            | ConfigsCommand::Edit(_)
            | ConfigsCommand::View(_)
            | ConfigsCommand::CloseView
            | ConfigsCommand::Apply
            | ConfigsCommand::EditAgain
            | ConfigsCommand::Reread
            | ConfigsCommand::Discard
            | ConfigsCommand::Dismiss => {}
        }
    }

    /// A plan the Console cannot take now is refused, as the apply service would.
    pub(super) fn act_configs(&mut self, act: Option<Act>) {
        match act {
            Some(Act::Run(plan)) => {
                if !self.run_owned(plan, Owner::Configs) {
                    let next = self.services.configs.plan_refused();
                    self.act_configs(next);
                }
            }
            Some(Act::Cancel) => self.to_service(ApplyCommand::Cancel),
            None => {}
        }
    }
}
