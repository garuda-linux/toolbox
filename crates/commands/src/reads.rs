//! Which commands read under `--json`. Their errors become objects on
//! standard error once the exit code is known, a change has its JSON Lines.

#[cfg(test)]
mod tests;

use toolbox_cli::args::{
    AppsCommand, BootCommand, BootRepairCommand, Command, ConfigsCommand, DkmsCommand,
    DriversCommand, Group, GroupsCommand, GuideCommand, KernelsCommand, LocalesCommand,
    MaintenanceCommand, NetworkCommand, PackagesCommand, PacksCommand, QueueCommand, ResetArgs,
    ResetList, ServicesCommand, SettingsCommand, SetupCommand, ToggleCommand, TweaksCommand,
    UsersCommand,
};

pub const fn json_read(command: &Command) -> bool {
    matches!(read_json(command), Some(true))
}

/// `Some(json)` for a command that only reads, with whether it reads as
/// JSON. The pager and the JSON errors both go by this one table.
#[expect(
    clippy::too_many_lines,
    reason = "one table with an arm per read, so a new command must say whether it reads"
)]
pub const fn read_json(command: &Command) -> Option<bool> {
    Some(match command {
        Command::Status(args) => args.read.json,
        Command::Services(ServicesCommand::List(list)) => list.read.json,
        Command::Services(ServicesCommand::Show(show)) => show.read.json,
        Command::Toggle(ToggleCommand::Show(show)) => show.read.json,
        Command::Toggle(ToggleCommand::List(list)) => list.read.json,
        Command::Apps(AppsCommand::List(list)) => list.read.json,
        Command::Kernels(KernelsCommand::List(list)) => list.read.json,
        Command::Locales(LocalesCommand::List(list)) => list.read.json,
        Command::Users(UsersCommand::List(list)) => list.read.json,
        Command::Maintenance(
            MaintenanceCommand::List(list)
            | MaintenanceCommand::Reset(ResetArgs {
                list: Some(ResetList::List(list)),
                entries: _,
                confirm: _,
            }),
        )
        | Command::Dkms(DkmsCommand::List(list))
        | Command::Locales(LocalesCommand::Packs(PacksCommand::List(list)))
        | Command::Groups(GroupsCommand::List(list))
        | Command::Configs(ConfigsCommand::List(list))
        | Command::Network(Group(NetworkCommand::Drivers(DriversCommand::List(list))))
        | Command::Setup(SetupCommand::List(list))
        | Command::Tweaks(Group(TweaksCommand::List(list)))
        | Command::Settings(Group(SettingsCommand::List(list)))
        | Command::Guide(Group(GuideCommand::List(list))) => list.json,
        Command::Doctor(read)
        | Command::Pages(read)
        | Command::Boot(Group(BootCommand::Show(read) | BootCommand::Entries(read)))
        | Command::Network(Group(
            NetworkCommand::Show(read) | NetworkCommand::PublicAddress(read),
        ))
        | Command::BootRepair(BootRepairCommand::Show(read))
        | Command::Queue(Group(QueueCommand::Show(read))) => read.json,
        Command::Settings(Group(SettingsCommand::Get(args))) => args.read.json,
        Command::Packages(PackagesCommand::Search(search)) => search.read.json,
        Command::Packages(PackagesCommand::Show(show)) => show.read.json,
        Command::Guide(Group(GuideCommand::Show(show))) => show.read.json,
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
        | Command::Tweaks(_)
        | Command::BootRepair(_)
        | Command::Setup(_)
        | Command::Settings(_)
        | Command::Queue(_)
        | Command::Completions(_)
        | Command::Help(_)
        | Command::Update(_) => return None,
    })
}
