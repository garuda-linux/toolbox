//! What the app does with a click on the Packages page and with the
//! page's reads. Installs and removals wait in the queue.

use toolbox_core::features::packages::aur;
use toolbox_core::features::packages::queue::toggle;

use super::App;
use crate::command::PackagesCommand;
use crate::page::PageId;
use crate::pages::PackagesRefusal;

impl App {
    pub(super) fn run_packages(&mut self, command: PackagesCommand) {
        match command {
            PackagesCommand::Refresh => {
                self.packages.refresh();
                self.read_system();
                self.status.refresh();
                self.facts.refresh();
            }
            PackagesCommand::Toggle { package, wish } => {
                let snapshot = self.current_snapshot();
                let result = self.queue.edit(&snapshot, |changes, snapshot| {
                    toggle(&package, wish, snapshot, changes)
                });

                self.refusals.packages = result
                    .err()
                    .map(|refusal| PackagesRefusal { refusal, package });
            }
            PackagesCommand::ToggleAur { package, wish } => {
                let snapshot = self.current_snapshot();
                let result = self.queue.edit(&snapshot, |changes, snapshot| {
                    aur::toggle(&package, wish, snapshot, changes)
                });

                self.refusals.packages = result
                    .err()
                    .map(|refusal| PackagesRefusal { refusal, package });
            }
            PackagesCommand::Show(package) => self.packages.show(&package),
            PackagesCommand::OpenAur => self.packages.open_aur(),
            PackagesCommand::DismissRefusal => self.refusals.packages = None,
        }
    }

    pub(super) fn take_packages(&mut self) {
        if self.state.page == PageId::Packages {
            self.packages.open();
        }

        self.packages.take();
        if self.packages.aur_landed() {
            self.combine();
            self.queue.meet(&self.seen);
        }
    }
}
