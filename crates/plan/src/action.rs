//! The closed set of things the app may change. Root actions run in the
//! helper, user actions in the app. Each carries validated names only.

mod accounts;
mod boot;
mod network;
mod packages;
mod repair;
mod settings;
mod targets;
#[cfg(test)]
pub mod testing;
#[cfg(test)]
mod tests;
mod unit;
mod update;
mod user;

pub use repair::{boot_loader_steps, describe_loader};
use serde::{Deserialize, Serialize};
pub use targets::{EmptyTargetList, TargetList};
pub use unit::{UnitOperation, UnitScope};
pub use update::{RemoteAction, Upgrader};
pub use user::UserAction;

use self::packages::{install, refresh_keyring, remote, remove, remove_orphans, upgrade};
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::sdboot::LoaderChange;
use crate::{
    ConfigWrite, DkmsBuild, DnsProvider, FullName, GarudaUpdateFlag, GroupName, GrubSettings,
    HomeChange, HomeFolder, InstalledSystem, KernelModule, KernelParameters, LocaleEntry,
    LocaleName, Mirrorlist, NewUser, PackageName, Password, PlymouthTheme, RepairWork, ShellPath,
    UnitName, UserName, WifiBackend,
};

/// Work that needs root. Runs in `garuda-toolbox-helper`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RootAction {
    InstallPackages {
        packages: TargetList<PackageName>,
    },
    RemovePackages {
        packages: TargetList<PackageName>,
    },
    UpgradeSystem {
        upgrader: Upgrader,
    },
    CleanPackageCache,
    RemoveOrphans,
    RefreshKeyring,
    RemoveDatabaseLock,
    GarudaUpdateRemote {
        action: RemoteAction,
    },
    GarudaHealthFix,
    /// A root shell inside the installed system, from the live ISO. Runs
    /// until the user leaves it in the Console.
    ChrootInstallation,
    /// Replaces the Chaotic-AUR mirrorlist, keeping the old one beside it.
    InstallMirrorlist {
        mirrorlist: Mirrorlist,
    },
    EnableUnits {
        units: TargetList<UnitName>,
    },
    DisableUnits {
        units: TargetList<UnitName>,
    },
    ControlUnits {
        operation: UnitOperation,
        units: TargetList<UnitName>,
    },
    MaskUnits {
        units: TargetList<UnitName>,
    },
    UnmaskUnits {
        units: TargetList<UnitName>,
    },
    AddToGroup {
        user: UserName,
        group: GroupName,
    },
    RemoveFromGroup {
        user: UserName,
        group: GroupName,
    },
    /// The helper refuses a shell that `/etc/shells` does not list.
    SetShell {
        user: UserName,
        shell: ShellPath,
    },
    SetDns {
        provider: DnsProvider,
    },
    SetHblock {
        enabled: bool,
    },
    /// Leaves NetworkManager started even when a step in between fails.
    SetWifiBackend {
        backend: WifiBackend,
    },
    SetGarudaUpdateFlag {
        flag: GarudaUpdateFlag,
        enabled: bool,
    },
    /// Switches one line of `/etc/locale.gen`. One
    /// [`RootAction::GenerateLocales`] follows all of them.
    SetLocaleEnabled {
        locale: LocaleEntry,
        enabled: bool,
    },
    GenerateLocales,
    /// The helper refuses a locale that `/etc/locale.gen` has not enabled.
    SetSystemLocale {
        locale: LocaleName,
    },
    InstallDkmsModule {
        build: DkmsBuild,
    },
    /// `useradd`. The account has no password until a
    /// [`RootAction::SetPassword`] sets one.
    CreateUser {
        user: NewUser,
    },
    /// The password is no part of the encoded action: it travels on its
    /// own line after the request (see [`Password::wire_line`]) and reaches
    /// `chpasswd` on its standard input.
    SetPassword {
        user: UserName,
        #[serde(skip)]
        password: Option<Password>,
    },
    /// The helper refuses the account with user id 0.
    DeleteUser {
        user: UserName,
        home: HomeFolder,
    },
    /// Locks or unlocks the password. The helper refuses to lock the
    /// account with user id 0.
    SetUserLocked {
        user: UserName,
        locked: bool,
    },
    SetFullName {
        user: UserName,
        full_name: FullName,
    },
    /// The helper refuses the account with user id 0.
    SetHomeFolder {
        user: UserName,
        change: HomeChange,
    },
    CreateGroup {
        group: GroupName,
    },
    /// The helper refuses a system group (group id below 1000).
    DeleteGroup {
        group: GroupName,
    },
    /// Replaces a config file with the text the user edited in a copy.
    /// The helper refuses when the file is not the version `original`
    /// marks anymore, and keeps the previous version in its backup folder.
    WriteConfigFile {
        write: ConfigWrite,
    },
    /// `modprobe` with NetworkManager stopped meanwhile and started again
    /// whatever happens.
    LoadModule {
        module: KernelModule,
    },
    UnloadModule {
        module: KernelModule,
    },
    /// Changes only the app's own modprobe file. It takes effect the next
    /// time the kernel wants to load the module by itself.
    SetModuleBlacklisted {
        module: KernelModule,
        blacklisted: bool,
    },
    /// Sets variables of `/etc/default/grub`, every other byte kept. One
    /// [`RootAction::GenerateGrubConfig`] follows it.
    SetGrubSettings {
        settings: GrubSettings,
    },
    SetLoaderSettings {
        settings: LoaderChange,
    },
    GenerateGrubConfig,
    /// Also builds the initramfs again, which holds the theme.
    SetPlymouthTheme {
        theme: PlymouthTheme,
    },
    /// One [`RootAction::RebuildUkis`] follows it.
    SetKernelCmdline {
        parameters: KernelParameters,
    },
    /// Builds nothing: on systemd-boot the initramfs is part of the UKI,
    /// so [`RootAction::RebuildUkis`] builds both.
    SelectPlymouthTheme {
        theme: PlymouthTheme,
    },
    RebuildUkis {
        initramfs: bool,
    },
    /// Mounts an installed system from the live ISO, installs GRUB or
    /// generates its menu inside it and unmounts everything again. The
    /// passphrase of a locked root is no part of the encoded action, like
    /// the password of [`RootAction::SetPassword`].
    RepairBoot {
        system: InstalledSystem,
        work: RepairWork,
        #[serde(skip)]
        passphrase: Option<Password>,
    },
}

impl RootAction {
    pub const fn needs_secret(&self) -> bool {
        if let Self::RepairBoot {
            system,
            work: _,
            passphrase: _,
        } = self
        {
            return system.is_locked();
        }

        matches!(
            self,
            Self::SetPassword {
                user: _,
                password: _
            }
        )
    }

    /// The secret the action carries, `None` when it needs none or when it
    /// was decoded from a request and its secret line is still to come.
    pub const fn secret(&self) -> Option<&Password> {
        if self.needs_secret()
            && let Self::SetPassword {
                user: _,
                password: secret,
            }
            | Self::RepairBoot {
                system: _,
                work: _,
                passphrase: secret,
            } = self
        {
            secret.as_ref()
        } else {
            None
        }
    }

    pub fn attach_secret(&mut self, secret: Password) -> bool {
        if self.needs_secret()
            && let Self::SetPassword {
                user: _,
                password: slot,
            }
            | Self::RepairBoot {
                system: _,
                work: _,
                passphrase: slot,
            } = self
        {
            *slot = Some(secret);
            true
        } else {
            false
        }
    }

    pub fn procedure(&self) -> Vec<Invocation> {
        match self {
            Self::InstallPackages { packages } => vec![run(install(packages))],
            Self::RemovePackages { packages } => vec![run(remove().args(names(packages)))],
            Self::UpgradeSystem { upgrader } => vec![run(upgrade(*upgrader))],
            Self::CleanPackageCache => vec![run(Command::new(Program::Paccache).arg("-ruk0"))],
            Self::RemoveOrphans => vec![remove_orphans()],
            Self::RefreshKeyring => vec![run(refresh_keyring())],
            Self::RemoveDatabaseLock => vec![Invocation::Builtin(Builtin::RemoveDatabaseLock)],
            Self::GarudaUpdateRemote { action } => vec![run(remote(*action))],
            Self::GarudaHealthFix => vec![run(Command::new(Program::GarudaHealth).arg("--fix"))],
            Self::ChrootInstallation => vec![run(Command::new(Program::GarudaChroot).arg("-a"))],
            Self::InstallMirrorlist { mirrorlist } => install_mirrorlist(mirrorlist),
            Self::EnableUnits { units } => on_units(["enable", "--now"], units),
            Self::DisableUnits { units } => on_units(["disable", "--now"], units),
            Self::ControlUnits { operation, units } => on_units([operation.as_str()], units),
            Self::MaskUnits { units } => on_units(["mask"], units),
            Self::UnmaskUnits { units } => on_units(["unmask"], units),
            Self::AddToGroup { user, group } => vec![run(gpasswd("-a", user, group))],
            Self::RemoveFromGroup { user, group } => vec![run(gpasswd("-d", user, group))],
            Self::SetShell { user, shell } => settings::set_shell(user, shell),
            Self::SetDns { provider } => settings::set_dns(*provider),
            Self::SetHblock { enabled } => settings::set_hblock(*enabled),
            Self::SetWifiBackend { backend } => settings::set_wifi_backend(*backend),
            Self::SetGarudaUpdateFlag { flag, enabled } => {
                settings::set_garuda_update_flag(*flag, *enabled)
            }
            Self::SetLocaleEnabled { locale, enabled } => {
                settings::set_locale_enabled(locale, *enabled)
            }
            Self::GenerateLocales => vec![run(Command::new(Program::LocaleGen))],
            Self::SetSystemLocale { locale } => settings::set_system_locale(locale),
            Self::InstallDkmsModule { build } => vec![run(build.command())],
            Self::CreateUser { user } => accounts::create_user(user),
            Self::SetPassword { user, password } => accounts::set_password(user, password.as_ref()),
            Self::DeleteUser { user, home } => accounts::delete_user(user, *home),
            Self::SetUserLocked { user, locked } => accounts::set_locked(user, *locked),
            Self::SetFullName { user, full_name } => accounts::set_full_name(user, full_name),
            Self::SetHomeFolder { user, change } => accounts::set_home_folder(user, change),
            Self::CreateGroup { group } => accounts::create_group(group),
            Self::DeleteGroup { group } => accounts::delete_group(group),
            Self::WriteConfigFile { write } => vec![write.invocation()],
            Self::LoadModule { module } => network::load_module(module),
            Self::UnloadModule { module } => network::unload_module(module),
            Self::SetModuleBlacklisted {
                module,
                blacklisted,
            } => network::set_module_blacklisted(module, *blacklisted),
            Self::SetGrubSettings { settings } => boot::set_grub_settings(settings),
            Self::SetLoaderSettings { settings } => boot::set_loader_settings(settings),
            Self::GenerateGrubConfig => boot::generate_grub_config(),
            Self::SetPlymouthTheme { theme } => boot::set_plymouth_theme(theme),
            Self::SetKernelCmdline { parameters } => boot::set_kernel_cmdline(parameters),
            Self::SelectPlymouthTheme { theme } => vec![boot::plymouth_theme(theme)],
            Self::RebuildUkis { initramfs } => boot::rebuild_ukis(*initramfs),
            Self::RepairBoot {
                system,
                work,
                passphrase,
            } => repair::repair_boot(system, work, passphrase.as_ref()),
        }
    }
}

const fn run(command: Command) -> Invocation {
    Invocation::Run(command)
}

fn names<T>(targets: &TargetList<T>) -> impl Iterator<Item = String> + '_
where
    T: AsRef<str>,
{
    targets.iter().map(|target| target.as_ref().to_owned())
}

fn install_mirrorlist(mirrorlist: &Mirrorlist) -> Vec<Invocation> {
    vec![Invocation::Builtin(Builtin::InstallMirrorlist(
        mirrorlist.clone(),
    ))]
}

fn systemctl<const N: usize>(args: [&str; N]) -> Command {
    Command::new(Program::Systemctl).args(args)
}

/// The `--` keeps a unit name from being read as an option.
fn on_units<const N: usize>(verbs: [&str; N], units: &TargetList<UnitName>) -> Vec<Invocation> {
    vec![run(systemctl(verbs).arg("--").args(names(units)))]
}

fn gpasswd(flag: &str, user: &UserName, group: &GroupName) -> Command {
    Command::new(Program::Gpasswd).args([flag, user.as_str(), group.as_str()])
}
