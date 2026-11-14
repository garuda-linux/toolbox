//! The Config Files page's clicks. A step that runs in the Console goes
//! back to App as an [`super::Act`]. A refused open, edit or view stays on the
//! page until dismissed or the next one is taken.

#[cfg(test)]
mod tests;

use toolbox_core::features::configs::Refusal;

use super::Configs;
use crate::command::{AppCommand, ConfigsCommand};
use crate::page_service::{Answer, Handled};
use crate::refused::Refused;

impl Configs {
    pub fn handle(&mut self, command: &AppCommand) -> Handled {
        let AppCommand::Configs(command) = command else {
            return Handled::default();
        };

        let act = match command {
            ConfigsCommand::Refresh => {
                self.refresh();
                None
            }
            ConfigsCommand::Open(key) => {
                let result = self.open_home(key);
                self.refuse(result, key);
                None
            }
            ConfigsCommand::Edit(key) => {
                let (result, act) = match self.edit(key) {
                    Ok(act) => (Ok(()), act),
                    Err(code) => (Err(code), None),
                };

                self.refuse(result, key);
                act
            }
            ConfigsCommand::View(key) => {
                let result = self.view(key);
                self.refuse(result, key);
                None
            }
            ConfigsCommand::CloseView => {
                self.close_view();
                None
            }
            ConfigsCommand::Apply => self.apply(),
            ConfigsCommand::EditAgain => self.edit_again(),
            ConfigsCommand::Reread => self.reread(),
            ConfigsCommand::Discard => self.discard(),
            ConfigsCommand::Dismiss => {
                self.dismiss();
                None
            }
            // The Console is App's.
            ConfigsCommand::ShowConsole => return Handled::default(),
        };

        Handled::answers(
            act.map(Answer::Configs)
                .map(AppCommand::Answer)
                .into_iter()
                .collect(),
        )
    }

    fn refuse(&mut self, result: Result<(), Refusal>, subject: &str) {
        let refusal = result
            .err()
            .map(|refusal| Refused::new(refusal, subject.to_owned()));

        self.set_refusal(refusal);
    }
}
