//! Which change commands `queue add` takes, and their wishes built onto
//! the queue by the same functions the commands use. What the window runs
//! at once (a start of a service, a password, an update) never waits in
//! its queue, so it does not wait here either.

#[cfg(test)]
mod tests;

use toolbox_cli::args::{
    AppsCommand, BootCommand, BootSet, Command, DkmsBuild, DkmsCommand, DriversCommand, EditUser,
    EntryArgs, Group, GroupArgs, GroupsCommand, KernelArgs, KernelsCommand, LocaleArgs,
    LocalesCommand, MaintenanceCommand, ModuleArgs, NetworkCommand, PackageArgs, PackageInstall,
    PackagesCommand, ParamArgs, ParamCommand, RemoveUser, TaskArgs, ToggleCommand, TweaksCommand,
    UnitArgs, UnitChange, UserArgs, UsersCommand,
};
use toolbox_core::Toggle;
use toolbox_core::features::maintenance::page::OPERATION_KEYS;
use toolbox_core::features::services::critical;

use crate::change::KernelChange;

#[derive(Debug)]
pub enum Queueable<'a> {
    Toggle(&'a EntryArgs, Toggle),
    App(&'a EntryArgs, Toggle),
    Unit(UnitChange, &'a UnitArgs),
    Kernel(&'a KernelArgs, KernelChange),
    Package(&'a PackageArgs, Toggle),
    /// Repository installs, or AUR builds with `--aur`.
    Install(&'a PackageInstall),
    Task(&'a TaskArgs),
    Dkms(&'a DkmsBuild),
    Locale(&'a LocaleArgs, Toggle),
    SystemLocale(&'a LocaleArgs),
    EditUser(&'a EditUser),
    Lock(&'a UserArgs, Toggle),
    RemoveUser(&'a RemoveUser),
    Group(&'a GroupArgs, Toggle),
    BootSet(&'a BootSet),
    Parameter(&'a ParamArgs, Toggle),
    Driver(&'a ModuleArgs, Toggle),
    Tweak(&'a TweaksCommand),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotQueued {
    /// The window runs it at once, beside the queue.
    AtOnce,
    /// It changes nothing.
    Reads,
    /// `--yes`, `--dry-run` and `--json` belong to `queue apply`.
    Flag,
}

impl Queueable<'_> {
    /// The name to type before the change joins the queue, for one that
    /// wipes a home or masks what the session lives on, as the command
    /// that runs it now asks.
    pub fn typed(&self) -> Option<String> {
        match self {
            Self::Unit(UnitChange::Mask, args) if critical(&args.unit) => {
                Some(args.unit.to_string())
            }
            Self::RemoveUser(args) if args.delete_home => Some(args.user.to_string()),
            Self::Toggle(..)
            | Self::App(..)
            | Self::Unit(..)
            | Self::Kernel(..)
            | Self::Package(..)
            | Self::Install(_)
            | Self::Task(_)
            | Self::Dkms(_)
            | Self::Locale(..)
            | Self::SystemLocale(_)
            | Self::EditUser(_)
            | Self::Lock(..)
            | Self::RemoveUser(_)
            | Self::Group(..)
            | Self::BootSet(_)
            | Self::Parameter(..)
            | Self::Driver(..)
            | Self::Tweak(_) => None,
        }
    }
}

impl NotQueued {
    pub const fn words(self) -> (&'static str, &'static str) {
        match self {
            Self::AtOnce => (
                "this change runs at once, it never waits in the queue",
                "run it without `queue add`",
            ),
            Self::Reads => (
                "this command changes nothing to queue",
                "garuda-toolbox queue add apps install firefox",
            ),
            Self::Flag => (
                "--yes, --dry-run and --json belong to `queue apply`",
                "queue the change without them",
            ),
        }
    }
}
/// What `command` would queue.
///
/// # Errors
///
/// [`NotQueued`] for a command the window runs at once, one that only
/// reads, or one with the flags of a change run now.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per change command, a new command fails to compile until it is sorted in"
)]
pub fn queueable(command: &Command) -> Result<Queueable<'_>, NotQueued> {
    let confirm = command.confirm().ok_or(NotQueued::Reads)?;
    if confirm.yes || confirm.dry_run || confirm.json {
        return Err(NotQueued::Flag);
    }

    match command {
        Command::Toggle(ToggleCommand::On(args)) => Ok(Queueable::Toggle(args, Toggle::On)),
        Command::Toggle(ToggleCommand::Off(args)) => Ok(Queueable::Toggle(args, Toggle::Off)),
        Command::Apps(AppsCommand::Install(args)) => Ok(Queueable::App(args, Toggle::On)),
        Command::Apps(AppsCommand::Remove(args)) => Ok(Queueable::App(args, Toggle::Off)),
        Command::Packages(PackagesCommand::Install(args)) => Ok(Queueable::Install(args)),
        Command::Packages(PackagesCommand::Remove(args)) => {
            Ok(Queueable::Package(args, Toggle::Off))
        }
        Command::Services(services) => match services.change() {
            Ok((
                change @ (UnitChange::Enable
                | UnitChange::Disable
                | UnitChange::Mask
                | UnitChange::Unmask),
                args,
            )) => Ok(Queueable::Unit(change, args)),
            Ok(_) | Err(_) => Err(NotQueued::AtOnce),
        },
        Command::Kernels(KernelsCommand::Install(args)) => {
            Ok(Queueable::Kernel(args, KernelChange::Install))
        }
        Command::Kernels(KernelsCommand::Remove(args)) => {
            Ok(Queueable::Kernel(args, KernelChange::Remove))
        }
        Command::Kernels(KernelsCommand::Headers(args)) => {
            Ok(Queueable::Kernel(args, KernelChange::Headers))
        }
        Command::Maintenance(MaintenanceCommand::Run(args))
            if OPERATION_KEYS.contains(&args.task.as_str()) =>
        {
            Ok(Queueable::Task(args))
        }
        Command::Dkms(DkmsCommand::Build(args)) => Ok(Queueable::Dkms(args)),
        Command::Locales(LocalesCommand::Enable(args)) => Ok(Queueable::Locale(args, Toggle::On)),
        Command::Locales(LocalesCommand::Disable(args)) => Ok(Queueable::Locale(args, Toggle::Off)),
        Command::Locales(LocalesCommand::System(args)) => Ok(Queueable::SystemLocale(args)),
        Command::Users(UsersCommand::Edit(args)) => Ok(Queueable::EditUser(args)),
        Command::Users(UsersCommand::Lock(args)) => Ok(Queueable::Lock(args, Toggle::On)),
        Command::Users(UsersCommand::Unlock(args)) => Ok(Queueable::Lock(args, Toggle::Off)),
        Command::Users(UsersCommand::Remove(args)) => Ok(Queueable::RemoveUser(args)),
        Command::Groups(GroupsCommand::Add(args)) => Ok(Queueable::Group(args, Toggle::On)),
        Command::Groups(GroupsCommand::Remove(args)) => Ok(Queueable::Group(args, Toggle::Off)),
        Command::Boot(Group(BootCommand::Set(args))) => Ok(Queueable::BootSet(args)),
        Command::Boot(Group(BootCommand::Param(ParamCommand::Add(args)))) => {
            Ok(Queueable::Parameter(args, Toggle::On))
        }
        Command::Boot(Group(BootCommand::Param(ParamCommand::Remove(args)))) => {
            Ok(Queueable::Parameter(args, Toggle::Off))
        }
        Command::Network(Group(NetworkCommand::Drivers(DriversCommand::Blacklist(args)))) => {
            Ok(Queueable::Driver(args, Toggle::On))
        }
        Command::Network(Group(NetworkCommand::Drivers(DriversCommand::Allow(args)))) => {
            Ok(Queueable::Driver(args, Toggle::Off))
        }
        Command::Tweaks(Group(
            tweak @ (TweaksCommand::Dns(_)
            | TweaksCommand::WifiBackend(_)
            | TweaksCommand::Hblock(_)
            | TweaksCommand::UpdateFlag(_)),
        )) => Ok(Queueable::Tweak(tweak)),
        Command::Status(_)
        | Command::Doctor(_)
        | Command::Pages(_)
        | Command::Guide(_)
        | Command::Diagnostics(_)
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
        | Command::Update(_)
        | Command::Queue(_)
        | Command::Completions(_)
        | Command::Help(_) => Err(NotQueued::AtOnce),
    }
}
