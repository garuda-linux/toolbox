//! What the app does with a click on Boot options and with the page's
//! background results.

use toolbox_core::domains::BootLoader;
use toolbox_core::features::boot::edit::{Edit, edit};
use toolbox_core::features::boot::grub;

use super::App;
use crate::boot::{BootRefusal, FileKind};
use crate::command::BootCommand;
use crate::page::PageId;

impl App {
    pub(super) fn run_boot(&mut self, command: BootCommand) {
        match command {
            BootCommand::Edit(change) => self.edit_boot(change),
            BootCommand::DismissRefusal => self.boot.set_refusal(None),
            BootCommand::Refresh => {
                self.boot.refresh();
                self.read_system();
            }
            BootCommand::Choose {
                kind,
                title,
                filter,
            } => self.boot.choose(kind, &title, &filter),
            BootCommand::Propose(kernel) => {
                self.boot.propose(Some(kernel));
                self.state.page = PageId::Boot;
            }
            BootCommand::DropProposal => self.boot.propose(None),
        }
    }

    /// The menu's first read waits for the page, and a chosen file is
    /// queued as soon as it comes.
    pub(super) fn take_boot(&mut self) {
        if self.state.page == PageId::Boot {
            let snapshot = self.current_snapshot();
            let sdboot = snapshot.part::<BootLoader>() == Some(&BootLoader::SystemdBoot);

            self.boot.open(sdboot || grub(&snapshot).is_ok());
        }

        if let Some(picked) = self.boot.take() {
            let change = match picked.kind {
                FileKind::Background => Edit::Background(picked.path),
                FileKind::Theme => Edit::Theme(picked.path),
            };

            self.edit_boot(change);
        }
    }

    /// A refusal stays on the page until dismissed or the next edit is taken.
    fn edit_boot(&mut self, change: Edit) {
        let value = change.value();
        let snapshot = self.current_snapshot();
        let result = self.queue.edit(&snapshot, |changes, snapshot| {
            edit(change, changes, snapshot)
        });

        self.boot
            .set_refusal(result.err().map(|problem| BootRefusal {
                code: problem.code(),
                value,
            }));
    }
}
