//! The Packages page's installs and removals, which wait in the queue. A
//! refused one stays on the page until dismissed or the next is taken.

#[cfg(test)]
mod tests;

use toolbox_core::features::packages::aur;
use toolbox_core::features::packages::queue::toggle;
use toolbox_core::{ChangeSet, Snapshot};

use super::{Packages, PackagesRefusal};
use crate::command::{AppCommand, PackagesCommand};
use crate::page_service::Handled;
use crate::refused::queue_or_keep;

impl Packages {
    pub const fn refusal(&self) -> Option<&PackagesRefusal> {
        self.refusal.as_ref()
    }

    pub fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        let AppCommand::Packages(command) = command else {
            return Handled::default();
        };

        match command {
            PackagesCommand::Toggle { package, wish } => {
                queue_or_keep(&mut self.refusal, changes, |changes| {
                    toggle(package, *wish, snapshot, changes)
                })
            }
            PackagesCommand::ToggleAur { package, wish } => {
                queue_or_keep(&mut self.refusal, changes, |changes| {
                    aur::toggle(package, *wish, snapshot, changes)
                })
            }
            PackagesCommand::DismissRefusal => {
                self.refusal = None;
                Handled::taken()
            }
            PackagesCommand::Refresh | PackagesCommand::Show(_) | PackagesCommand::OpenAur => {
                Handled::default()
            }
        }
    }
}
