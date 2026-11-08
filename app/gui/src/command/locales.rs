use toolbox_plan::{LocaleEntry, LocaleName, PackageName};

use super::{AppCommand, Command};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocaleCommand {
    Toggle { entry: LocaleEntry, on: bool },
    System(LocaleName),
    TogglePack { package: PackageName, on: bool },
    AddMissingPacks,
    DismissRefusal,
    Refresh,
}

impl From<LocaleCommand> for Command {
    fn from(command: LocaleCommand) -> Self {
        Self::App(AppCommand::Locale(command))
    }
}
