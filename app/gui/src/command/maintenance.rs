//! A click on the Maintenance page, or an entry of it picked in the palette.

use toolbox_core::Toggle;
use toolbox_core::features::maintenance::Operation;
use toolbox_core::features::maintenance::page::{PageTool, Repair};

use super::{AppCommand, Command};
use crate::pages::maintenance::Tab;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaintenanceCommand {
    /// Runs the operation at once in the Console.
    RunOperation(Operation),
    /// Runs the repair at once in the Console.
    RunRepair(Repair),
    /// Queues operation `operation` for the next apply, or takes it back.
    Queue {
        operation: Operation,
        on: Toggle,
    },
    /// Opens the tool, or installs it first and opens it then.
    OpenTool(PageTool),
    /// Installs every package a greyed repair needs, in one plan.
    InstallMissing,
    DismissProblem,
    /// Stops the ranking of the Chaotic-AUR mirrors under way.
    CancelRanking,
    /// Chooses settings `entry` for the reset, or drops it.
    Choose {
        entry: String,
        on: Toggle,
    },
    /// Copies the chosen settings from `/etc/skel`, once the user agreed.
    Reset,
    /// Opens the page on the tab, for an entry picked in the palette.
    Reveal(Tab),
    Refresh,
}

impl From<MaintenanceCommand> for Command {
    fn from(command: MaintenanceCommand) -> Self {
        Self::App(AppCommand::Maintenance(command))
    }
}
