//! What the app does with a click on the Config Files page and with what
//! its edits ask for: the editor and the write run in the Console, a
//! cancel stops the editor. A refused click stays on the page until
//! dismissed or the next click is taken.

use toolbox_core::features::configs::Refusal;

use super::App;
use crate::apply::ApplyCommand;
use crate::command::ConfigsCommand;
use crate::configs::Act;
use crate::refused::Refused;

impl App {
    pub(super) fn run_configs(&mut self, command: ConfigsCommand) {
        let act = match command {
            ConfigsCommand::Refresh => {
                self.services.configs.refresh();
                None
            }
            ConfigsCommand::Open(key) => {
                let result = self.services.configs.open_home(&key);
                self.refuse_configs(result, key);
                None
            }
            ConfigsCommand::Edit(key) => {
                let (result, act) = match self.services.configs.edit(&key) {
                    Ok(act) => (Ok(()), act),
                    Err(code) => (Err(code), None),
                };

                self.refuse_configs(result, key);
                act
            }
            ConfigsCommand::View(key) => {
                let result = self.services.configs.view(&key);
                self.refuse_configs(result, key);
                None
            }
            ConfigsCommand::CloseView => {
                self.services.configs.close_view();
                None
            }
            ConfigsCommand::Apply => self.services.configs.apply(),
            ConfigsCommand::EditAgain => self.services.configs.edit_again(),
            ConfigsCommand::Reread => self.services.configs.reread(),
            ConfigsCommand::Discard => self.services.configs.discard(),
            ConfigsCommand::Dismiss => {
                self.services.configs.dismiss();
                None
            }
            ConfigsCommand::ShowConsole => {
                self.state.console_open = true;
                None
            }
            ConfigsCommand::DismissRefusal => None,
        };

        self.act_configs(act);
    }

    /// A plan the Console cannot take now is refused, as the apply service would.
    pub(super) fn act_configs(&mut self, act: Option<Act>) {
        match act {
            Some(Act::Run(plan)) => {
                if !self.run_at_once(plan) {
                    let next = self.services.configs.plan_refused();
                    self.act_configs(next);
                }
            }
            Some(Act::Cancel) => self.to_service(ApplyCommand::Cancel),
            None => {}
        }
    }

    fn refuse_configs(&mut self, result: Result<(), Refusal>, subject: String) {
        let refusal = result.err().map(|refusal| Refused::new(refusal, subject));
        self.services.configs.set_refusal(refusal);
    }
}
