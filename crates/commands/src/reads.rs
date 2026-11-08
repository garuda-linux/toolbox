//! Which commands read under `--json`. Their errors become objects on
//! standard error once the exit code is known, a change has its JSON Lines.

#[cfg(test)]
mod tests;

use toolbox_cli::args::{
    AppsCommand, BootCommand, BootRepairCommand, Command, ConfigsCommand, DkmsCommand,
    DriversCommand, GroupsCommand, GuideCommand, KernelsCommand, LocalesCommand,
    MaintenanceCommand, NetworkCommand, PackagesCommand, PacksCommand, QueueCommand,
    ServicesCommand, SettingsCommand, SetupCommand, ToggleCommand, UsersCommand,
};

pub const fn json_read(command: &Command) -> bool {
    matches!(read_json(command), Some(true))
}

/// `Some(json)` for a command that only reads, with whether it reads as
/// JSON. The pager and the JSON errors both go by this one table.
pub const fn read_json(command: &Command) -> Option<bool> {
    let json = match command {
        Command::Status(args) => args.read.json,
        Command::Services(ServicesCommand::List(list)) => list.read.json,
        Command::Toggle(ToggleCommand::List(list)) => list.read.json,
        Command::Apps(AppsCommand::List(list)) => list.read.json,
        Command::Kernels(KernelsCommand::List(list)) => list.read.json,
        Command::Locales(LocalesCommand::List(list)) => list.read.json,
        Command::Users(UsersCommand::List(list)) => list.read.json,
        Command::Maintenance(MaintenanceCommand::List(list))
        | Command::Dkms(DkmsCommand::List(list))
        | Command::Locales(LocalesCommand::Packs(PacksCommand::List(list)))
        | Command::Groups(GroupsCommand::List(list))
        | Command::Configs(ConfigsCommand::List(list))
        | Command::Network(NetworkCommand::Drivers(DriversCommand::List(list)))
        | Command::Setup(SetupCommand::List(list))
        | Command::Settings(SettingsCommand::List(list)) => list.json,
        Command::Doctor(read)
        | Command::Pages(read)
        | Command::Boot(BootCommand::Show(read) | BootCommand::Entries(read))
        | Command::Network(NetworkCommand::Show(read) | NetworkCommand::PublicAddress(read))
        | Command::BootRepair(BootRepairCommand::Show(read))
        | Command::Queue(QueueCommand::Show(read)) => read.json,
        Command::Settings(SettingsCommand::Get(args)) => args.read.json,
        Command::Packages(PackagesCommand::Search(search)) => search.read.json,
        Command::Packages(PackagesCommand::Show(show)) => show.read.json,
        Command::Guide(GuideCommand::List(list)) => list.read().json,
        Command::Guide(GuideCommand::Show(show)) => show.read.json,
        Command::Diagnostics(_)
        | Command::Services(_)
        | Command::Toggle(_)
        | Command::Apps(_)
        | Command::Packages(_)
        | Command::Kernels(_)
        | Command::Maintenance(_)
        | Command::Dkms(_)
        | Command::Locales(_)
        | Command::Users(_)
        | Command::Groups(_)
        | Command::Boot(_)
        | Command::Configs(_)
        | Command::Network(_)
        | Command::BootRepair(_)
        | Command::Setup(_)
        | Command::Settings(_)
        | Command::Queue(_)
        | Command::Completions(_)
        | Command::Help(_)
        | Command::Update(_) => return None,
    };

    Some(json)
}
