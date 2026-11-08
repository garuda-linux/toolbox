//! A click on the Maintenance page, or an entry of it picked in the palette.

use super::{AppCommand, Command};
use crate::pages::maintenance::Tab;

/// The ids are the page's: an operation, a repair or a tool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaintenanceCommand {
    /// Runs operation or repair `0` at once in the Console.
    RunNow(String),
    /// Queues operation `operation` for the next apply, or takes it back.
    Queue {
        operation: String,
        on: bool,
    },
    /// Opens tool `0`, or installs it first and opens it then.
    OpenTool(String),
    /// Installs garuda-update, which most repairs need.
    InstallGarudaUpdate,
    DismissProblem,
    /// Chooses settings `entry` for the reset, or drops it.
    Choose {
        entry: String,
        on: bool,
    },
    /// Copies the chosen settings from `/etc/skel`, once the user agreed.
    Reset,
    /// Opens the page on tab `0`, for an entry picked in the palette.
    Reveal(Tab),
    Refresh,
}

impl From<MaintenanceCommand> for Command {
    fn from(command: MaintenanceCommand) -> Self {
        Self::App(AppCommand::Maintenance(command))
    }
}
