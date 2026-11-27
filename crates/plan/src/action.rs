//! The closed set of things the app may change. Root actions run in the
//! helper, user actions in the app. Each carries validated names only.

mod accounts;
mod boot;
mod network;
mod packages;
mod procedure;
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

pub use repair::{LoaderRefusal, boot_loader_steps, describe_loader, loader_why};
use serde::{Deserialize, Serialize};
pub use targets::{EmptyTargetList, TargetList};
pub use unit::{UnitOperation, UnitScope};
pub use update::{RemoteAction, Upgrader};
pub use user::UserAction;

use crate::procedure::{Command, Invocation, Program};
use crate::sdboot::LoaderChange;
use crate::{
    BuildRun, ConfigWrite, DkmsBuild, DnsProvider, FullName, GarudaUpdateFlag, GroupName,
    GrubSettings, HomeChange, HomeFolder, InstallReason, InstalledSystem, KernelModule,
    KernelParameters, LocaleEntry, LocaleName, MergeResolve, Mirrorlist, NewUser, PackageName,
    Password, PlymouthTheme, RepairWork, ShellPath, Toggle, UnitName, UserName, WifiBackend,
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
    /// What an AUR build needs from the repositories.
    InstallDependencies {
        packages: TargetList<PackageName>,
    },
    InstallBuilt {
        run: BuildRun,
        packages: TargetList<PackageName>,
        reason: InstallReason,
    },
    UpgradeSystem {
        upgrader: Upgrader,
    },
    CleanPackageCache,
    RemoveOrphans,
    /// Removes the named packages that are installed as dependencies and
    /// that nothing requires, and no others. Never with `-s`.
    RemoveMakeDependencies {
        packages: TargetList<PackageName>,
    },
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
        #[serde(with = "crate::toggle::as_bool")]
        enabled: Toggle,
    },
    /// Leaves NetworkManager started even when a step in between fails.
    SetWifiBackend {
        backend: WifiBackend,
    },
    SetGarudaUpdateFlag {
        flag: GarudaUpdateFlag,
        #[serde(with = "crate::toggle::as_bool")]
        enabled: Toggle,
    },
    /// Switches one line of `/etc/locale.gen`. One
    /// [`RootAction::GenerateLocales`] follows all of them.
    SetLocaleEnabled {
        locale: LocaleEntry,
        #[serde(with = "crate::toggle::as_bool")]
        enabled: Toggle,
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
        #[serde(with = "crate::toggle::as_bool")]
        locked: Toggle,
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
    /// Removes a copy pacman left beside a configuration file, or puts it
    /// or a merged text in the file's place.
    ResolveMerge {
        merge: MergeResolve,
    },
    /// A snapper snapshot of `/` before the merges, where snapper is set up.
    SnapshotBeforeMerges,
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
        #[serde(with = "crate::toggle::as_bool")]
        blacklisted: Toggle,
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
}

fn names<T>(targets: &TargetList<T>) -> impl Iterator<Item = String> + '_
where
    T: AsRef<str>,
{
    targets.iter().map(|target| target.as_ref().to_owned())
}

const fn run(command: Command) -> Invocation {
    Invocation::Run(command)
}

fn systemctl<const N: usize>(args: [&str; N]) -> Command {
    Command::new(Program::Systemctl).args(args)
}

/// The `--` keeps a unit name from being read as an option.
fn on_units<const N: usize>(verbs: [&str; N], units: &TargetList<UnitName>) -> Vec<Invocation> {
    vec![run(systemctl(verbs).arg("--").args(names(units)))]
}
