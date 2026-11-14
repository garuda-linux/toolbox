//! The Boot options page's edits, which wait in the queue. A refused one
//! stays on the page until dismissed or the next edit is taken.

#[cfg(test)]
mod tests;

use toolbox_core::features::boot::edit::{Edit, edit};
use toolbox_core::features::boot::set_loader;
use toolbox_core::{ChangeSet, Snapshot};

use super::Boot;
use crate::command::{AppCommand, BootCommand};
use crate::page::PageId;
use crate::page_service::{Answer, Handled};
use crate::refused::{queue, queue_or_refuse};

impl Boot {
    pub fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        let AppCommand::Boot(command) = command else {
            return Handled::default();
        };

        match command {
            BootCommand::Edit(change) => self.edit(change.clone(), changes, snapshot),
            BootCommand::Loader(setting) => queue(changes, |changes| {
                set_loader(setting.clone(), changes, snapshot);
            }),
            BootCommand::Refresh => {
                self.refresh();
                Handled::answers(vec![AppCommand::ReadSystem])
            }
            BootCommand::Choose {
                kind,
                title,
                filter,
            } => {
                self.choose(*kind, title, filter);
                Handled::taken()
            }
            BootCommand::Propose(kernel) => {
                self.propose(Some(kernel.clone()));
                Handled::answers(vec![AppCommand::Answer(Answer::ShowPage(PageId::Boot))])
            }
            BootCommand::DropProposal => {
                self.propose(None);
                Handled::taken()
            }
        }
    }

    /// Also for a file the chooser picked, which comes from the page's
    /// own reads.
    pub fn edit(&mut self, change: Edit, changes: &ChangeSet, snapshot: &Snapshot) -> Handled {
        let value = change.value();
        queue_or_refuse(&mut self.refusal, value, changes, |changes| {
            edit(change, changes, snapshot).map_err(|problem| problem.code())
        })
    }
}
