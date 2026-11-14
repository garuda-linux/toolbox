//! What the app does with a click on the Packages page beyond its
//! service: Refresh also reads Home's status and facts.

use super::App;
use crate::command::PackagesCommand;

impl App {
    pub(super) fn run_packages(&mut self, command: &PackagesCommand) {
        match command {
            PackagesCommand::Refresh => {
                self.services.packages.refresh();
                self.read_system();
                self.services.home.refresh_packages();
            }
            // The service takes these, see `hand_to_services`.
            PackagesCommand::Show(_)
            | PackagesCommand::OpenAur
            | PackagesCommand::Toggle {
                package: _,
                wish: _,
            }
            | PackagesCommand::ToggleAur {
                package: _,
                wish: _,
            } => {}
        }
    }
}
