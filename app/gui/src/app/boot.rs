//! What the app does with a click on Boot options and with the page's
//! background results.

use toolbox_core::domains::LoaderSettingsDomain;
use toolbox_core::features::boot::edit::{Edit, edit};

use super::App;
use crate::boot::BootRefusal;
use crate::command::BootCommand;
use crate::page::PageId;

impl App {
    pub(super) fn run_boot(&mut self, command: BootCommand) {
        match command {
            BootCommand::Edit(change) => self.edit_boot(change),
            BootCommand::Loader(setting) => {
                let snapshot = self.current_snapshot();

                self.queue.edit(&snapshot, |changes, snapshot| {
                    LoaderSettingsDomain::want(changes, setting, snapshot);
                });
            }
            BootCommand::DismissRefusal => self.services.boot.set_refusal(None),
            BootCommand::Refresh => {
                self.services.boot.refresh();
                self.read_system();
            }
            BootCommand::Choose {
                kind,
                title,
                filter,
            } => self.services.boot.choose(kind, &title, &filter),
            BootCommand::Propose(kernel) => {
                self.services.boot.propose(Some(kernel));
                self.state.page = PageId::Boot;
            }
            BootCommand::DropProposal => self.services.boot.propose(None),
        }
    }

    /// A refusal stays on the page until dismissed or the next edit is taken.
    pub(super) fn edit_boot(&mut self, change: Edit) {
        let value = change.value();
        let snapshot = self.current_snapshot();
        let result = self.queue.edit(&snapshot, |changes, snapshot| {
            edit(change, changes, snapshot)
        });

        self.services
            .boot
            .set_refusal(result.err().map(|problem| BootRefusal {
                code: problem.code(),
                value,
            }));
    }
}
