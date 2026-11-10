//! The app learns from it before anything runs whether standard output is
//! JSON Lines only.

use super::{
    AppsCommand, BootCommand, BootRepairCommand, Command, ConfigsCommand, Confirm, DkmsCommand,
    DriversCommand, GroupsCommand, KernelsCommand, LocalesCommand, MaintenanceCommand,
    NetworkCommand, PackagesCommand, PacksCommand, ParamCommand, QueueCommand, ServicesCommand,
    SetupCommand, ToggleCommand, TweaksCommand, UsersCommand,
};

impl Command {
    /// The question of a change, `None` for a command that only reads.
    pub const fn confirm(&self) -> Option<Confirm> {
        match self {
            Self::Update(args) => Some(args.confirm),
            Self::Diagnostics(args) => Some(args.confirm),
            Self::Services(command) => services(command),
            Self::Toggle(ToggleCommand::On(args) | ToggleCommand::Off(args))
            | Self::Apps(AppsCommand::Install(args) | AppsCommand::Remove(args)) => {
                Some(args.confirm)
            }
            Self::Packages(PackagesCommand::Install(install)) => Some(install.confirm),
            Self::Packages(PackagesCommand::Remove(args)) => Some(args.confirm),
            Self::Kernels(command) => kernels(command),
            Self::Maintenance(MaintenanceCommand::Run(args)) => Some(args.confirm),
            Self::Maintenance(MaintenanceCommand::Reset(args)) if args.list.is_none() => {
                Some(args.confirm)
            }
            Self::Dkms(DkmsCommand::Build(args)) => Some(args.confirm),
            Self::Locales(command) => locales(command),
            Self::Users(command) => users(command),
            Self::Groups(GroupsCommand::Add(args) | GroupsCommand::Remove(args)) => {
                Some(args.confirm)
            }
            Self::Boot(command) => boot(command),
            Self::Configs(ConfigsCommand::Edit(args)) => Some(args.confirm),
            Self::Network(NetworkCommand::Drivers(
                DriversCommand::Blacklist(args)
                | DriversCommand::Allow(args)
                | DriversCommand::Load(args)
                | DriversCommand::Unload(args),
            )) => Some(args.confirm),
            Self::Tweaks(command) => tweaks(command),
            Self::BootRepair(BootRepairCommand::Regenerate(args)) => Some(args.confirm),
            Self::BootRepair(BootRepairCommand::Reinstall(args)) => Some(args.repair.confirm),
            Self::Setup(SetupCommand::Apply(args)) => Some(args.confirm),
            Self::Queue(QueueCommand::Apply(confirm)) => Some(*confirm),
            Self::Status(_)
            | Self::Doctor(_)
            | Self::Pages(_)
            | Self::Guide(_)
            | Self::Toggle(ToggleCommand::List(_))
            | Self::Apps(AppsCommand::List(_))
            | Self::Packages(PackagesCommand::Search(_) | PackagesCommand::Show(_))
            | Self::Maintenance(MaintenanceCommand::List(_) | MaintenanceCommand::Reset(_))
            | Self::Dkms(DkmsCommand::List(_))
            | Self::Groups(GroupsCommand::List(_))
            | Self::Configs(ConfigsCommand::List(_) | ConfigsCommand::Show(_))
            | Self::Network(_)
            | Self::BootRepair(BootRepairCommand::Show(_))
            | Self::Setup(SetupCommand::List(_))
            | Self::Settings(_)
            | Self::Queue(
                QueueCommand::Show(_)
                | QueueCommand::Add(_)
                | QueueCommand::Remove(_)
                | QueueCommand::Clear(_),
            )
            | Self::Completions(_)
            | Self::Help(_) => None,
        }
    }
}

const fn services(command: &ServicesCommand) -> Option<Confirm> {
    match command {
        ServicesCommand::List(_) => None,
        ServicesCommand::Start(args)
        | ServicesCommand::Stop(args)
        | ServicesCommand::Restart(args)
        | ServicesCommand::Reload(args)
        | ServicesCommand::Enable(args)
        | ServicesCommand::Disable(args)
        | ServicesCommand::Mask(args)
        | ServicesCommand::Unmask(args) => Some(args.confirm),
    }
}

const fn tweaks(command: &TweaksCommand) -> Option<Confirm> {
    match command {
        TweaksCommand::List(_) => None,
        TweaksCommand::Dns(args) => Some(args.confirm),
        TweaksCommand::WifiBackend(args) => Some(args.confirm),
        TweaksCommand::Hblock(args) => Some(args.confirm),
        TweaksCommand::UpdateFlag(args) => Some(args.confirm),
    }
}

const fn kernels(command: &KernelsCommand) -> Option<Confirm> {
    match command {
        KernelsCommand::List(_) => None,
        KernelsCommand::Install(args)
        | KernelsCommand::Remove(args)
        | KernelsCommand::Headers(args)
        | KernelsCommand::Default(args) => Some(args.confirm),
    }
}

const fn locales(command: &LocalesCommand) -> Option<Confirm> {
    match command {
        LocalesCommand::List(_) | LocalesCommand::Packs(PacksCommand::List(_)) => None,
        LocalesCommand::Enable(args)
        | LocalesCommand::Disable(args)
        | LocalesCommand::System(args) => Some(args.confirm),
        LocalesCommand::Packs(PacksCommand::Install(args) | PacksCommand::Remove(args)) => {
            Some(args.confirm)
        }
    }
}

const fn users(command: &UsersCommand) -> Option<Confirm> {
    match command {
        UsersCommand::List(_) => None,
        UsersCommand::Add(args) => Some(args.confirm),
        UsersCommand::Edit(args) => Some(args.confirm),
        UsersCommand::Password(args) => Some(args.confirm),
        UsersCommand::Lock(args) | UsersCommand::Unlock(args) => Some(args.confirm),
        UsersCommand::Remove(args) => Some(args.confirm),
    }
}

const fn boot(command: &BootCommand) -> Option<Confirm> {
    match command {
        BootCommand::Show(_) | BootCommand::Entries(_) => None,
        BootCommand::Set(args) => Some(args.confirm),
        BootCommand::Param(ParamCommand::Add(args) | ParamCommand::Remove(args)) => {
            Some(args.confirm)
        }
    }
}
