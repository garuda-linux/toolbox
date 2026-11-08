use toolbox_core::Toggle;
use toolbox_plan::PackageName;

use super::{AppCommand, Command};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackagesCommand {
    Refresh,
    /// Queues installing (`On`) or removing the package, or takes the
    /// queued wish back.
    Toggle {
        package: PackageName,
        wish: Toggle,
    },
    /// Queues building (`On`) an AUR package or removing it, or takes
    /// the queued wish back.
    ToggleAur {
        package: PackageName,
        wish: Toggle,
    },
    Show(PackageName),
    OpenAur,
    DismissRefusal,
}

impl From<PackagesCommand> for Command {
    fn from(command: PackagesCommand) -> Self {
        Self::App(AppCommand::Packages(command))
    }
}
