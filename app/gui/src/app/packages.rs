//! What the app does with a click on the Packages page beyond its
//! service, which queues the installs and removals.

use super::App;
use crate::command::PackagesCommand;

impl App {
    pub(super) fn run_packages(&mut self, command: PackagesCommand) {
        match command {
            PackagesCommand::Refresh => {
                self.services.packages.refresh();
                self.read_system();
                self.status.refresh();
                self.facts.refresh();
            }
            PackagesCommand::Show(package) => self.services.packages.show(&package),
            PackagesCommand::OpenAur => self.services.packages.open_aur(),
            PackagesCommand::Toggle {
                package: _,
                wish: _,
            }
            | PackagesCommand::ToggleAur {
                package: _,
                wish: _,
            }
            | PackagesCommand::DismissRefusal => {}
        }
    }
}
