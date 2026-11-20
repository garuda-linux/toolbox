use std::fmt;

use super::{
    CHAOTIC_MIRRORLIST, DATABASE_LOCK, FIRST_USER_ID, InitramfsRoot, SHELLS, Shown, initramfs, uki,
};
use crate::garuda_update::GARUDA_UPDATE_CONFIG;
use crate::grub::GRUB_DEFAULTS;
use crate::locale::{LOCALE_CONF, LOCALE_GEN};
use crate::network::MODPROBE_BLACKLIST;
use crate::repair::{ESP_FOLDER, REPAIR_ROOT, RepairNeed};
use crate::sdboot::{KERNEL_CMDLINE, LOADER_CONF, LoaderChange};
use crate::settings::{
    DNS_CONFIG, NETWORKMANAGER_CONF_D, OLD_WIFI_BACKEND_CONFIG, WIFI_BACKEND_CONFIG,
};
use crate::{
    AUR_BUILT, AUR_STORE, BuildRun, ConfigWrite, DnsProvider, GarudaUpdateFlag, GroupName,
    GrubSettings, HomeDirectory, HomeWrite, InstallReason, InstalledSystem, KernelModule,
    KernelParameters, LocaleEntry, LocaleName, MergeResolve, Mirrorlist, PARU_CLONE, PackageName,
    RepairWork, ShellPath, TargetList, Toggle, UserConfigEdit, UserName, WifiBackend,
};

/// Work the helper does itself instead of running a program, because it
/// needs a check no single program offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Builtin {
    /// Deletes `/var/lib/pacman/db.lck`, but only when no pacman process
    /// runs (a live transaction holds the lock for a reason).
    RemoveDatabaseLock,
    /// Replaces the Chaotic-AUR mirrorlist atomically without following
    /// symlinks, keeping the old file as `chaotic-mirrorlist.bak`.
    InstallMirrorlist(Mirrorlist),
    /// Fails unless the shell is listed in `/etc/shells`. Root may set any
    /// program as a login shell with chsh, so the helper checks the list
    /// that chsh only enforces for users.
    RequireListedShell(ShellPath),
    /// Writes NetworkManager's DNS drop-in for the provider, or removes it
    /// for the default.
    WriteDnsConfig(DnsProvider),
    /// Writes NetworkManager's Wi-Fi backend drop-in for iwd, or removes it
    /// (and the older tool's one) for `wpa_supplicant`.
    WriteWifiBackend(WifiBackend),
    /// Sets one switch in garuda-update's config, creating the file when
    /// it is missing.
    SetGarudaUpdateFlag { flag: GarudaUpdateFlag, on: bool },
    /// Turns one edit of a config file in the user's home on or off. Runs
    /// only in the app, as the user: the helper has no home to edit.
    EditUserConfig { edit: UserConfigEdit, on: bool },
    /// Switches one line of `/etc/locale.gen` on or off, every other byte
    /// kept.
    EditLocaleGen { entry: LocaleEntry, on: bool },
    /// Sets `LANG` in `/etc/locale.conf`. Fails unless `/etc/locale.gen`
    /// has a locale enabled that provides it: a `LANG` without its locale
    /// leaves every program with the C locale and warnings.
    SetSystemLocale(LocaleName),
    /// Fails for an account whose user id is outside [`FIRST_USER_ID`] to
    /// [`crate::LAST_USER_ID`], root and `nobody` among them, and for an
    /// account `/etc/passwd` does not list. The app's own guard is no
    /// protection once a request is on the wire.
    RequireOrdinaryUser(UserName),
    /// Fails when the folder exists. `usermod -d <new> -m` writes the new
    /// home into `/etc/passwd` before it refuses a folder that exists, so a
    /// move onto one would leave the account pointing at a foreign folder.
    /// A folder on the way may belong to the account of the user, once it
    /// is listed.
    RequireFreeHome(UserName, HomeDirectory),
    /// Fails below a folder everyone may write, where `useradd -m` makes the home.
    RequireSafeHome(HomeDirectory),
    /// Fails for a group with an id outside [`FIRST_USER_ID`] to
    /// [`crate::LAST_USER_ID`]: packages own those and files on disk refer
    /// to them.
    RequireUserGroup(GroupName),
    /// Replaces a config file with the text the user edited, keeping
    /// owner and mode and the previous version in the backup folder. Fails
    /// when the file no longer is the version the edit started from.
    WriteConfigFile(ConfigWrite),
    /// Removes a copy pacman left beside a configuration file, or puts it
    /// or a merged text in the file's place. Fails unless pacman lists the
    /// file as a backup file and both files are as the user saw them.
    ResolveMerge(MergeResolve),
    /// Runs [`crate::merge::snapshot`] where snapper has a configuration of
    /// the root file system, else does nothing.
    SnapshotBeforeMerges,
    /// [`Builtin::WriteConfigFile`] for a file in the user's home. Runs only
    /// in the app, as the user: the helper has no home to write.
    WriteHomeFile(HomeWrite),
    /// Copies the files `run` built for `packages` from the calling user's
    /// [`AUR_BUILT`] folder into [`AUR_STORE`], then `pacman -U`s the
    /// copies. Fails unless every folder on the way belongs to that user
    /// and no one else may write it, and every package has exactly one file.
    InstallBuilt {
        run: BuildRun,
        packages: TargetList<PackageName>,
        reason: InstallReason,
    },
    /// `makepkg` without `-s` in paru's clone of `base`, with `PKGDEST` at
    /// the run's folder. Runs only in the app, as the user.
    BuildAur { base: PackageName, run: BuildRun },
    /// Adds a module to the app's own modprobe file or takes it out again.
    /// No other modprobe file is ever written.
    EditModprobeBlacklist { module: KernelModule, on: bool },
    /// Sets variables of `/etc/default/grub` and keeps the old file
    /// beside it. Fails without the file: GRUB is not the boot loader then.
    EditGrubDefaults(GrubSettings),
    /// The ESP comes from the helper's own mount table, never from the
    /// request, so no request can point root at another file.
    EditLoaderConf(LoaderChange),
    /// The old file stays as `cmdline.bak`. A missing one is created.
    WriteKernelCmdline(KernelParameters),
    /// Makes the folder a boot repair mounts the installed system at,
    /// for root alone. Fails when something is mounted there already.
    PrepareRepairMount,
    /// Fails unless the file system mounted for a boot repair holds an
    /// installed system with GRUB and a kernel. Without this check a
    /// wrong partition would get a boot loader that starts nothing.
    RequireInstalledSystem(RepairNeed),
    /// Checks the devices of a boot repair again, right before
    /// `grub-install` writes to one. A stick plugged in or out since the
    /// preflight renumbers the kernel's device names.
    RecheckRepairDevices {
        system: InstalledSystem,
        work: RepairWork,
    },
    /// Runs the first of [`initramfs_rebuilds`] whose program is
    /// installed below `root`. Fails when none is.
    RebuildInitramfs { root: InitramfsRoot },
    /// Runs the first of [`uki_rebuilds`] that is installed below `root`.
    /// Fails when none is.
    RebuildUkis { root: InitramfsRoot },
    /// The helper looks at the mounted system and runs
    /// [`crate::boot_loader_steps`] for its boot loader.
    RepairBootLoader {
        system: InstalledSystem,
        work: RepairWork,
    },
}

impl fmt::Display for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_command(f, Shown::WithNotes)?;
        self.why().map_or(Ok(()), |why| write!(f, "  # {why}"))
    }
}

impl Builtin {
    /// The part's command as the Console shows it, without its note.
    pub fn command(&self) -> String {
        fmt::from_fn(|out| self.write_command(out, Shown::Commands)).to_string()
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one arm per builtin, a new builtin fails to compile until it names its command"
    )]
    pub(super) fn write_command(&self, out: &mut fmt::Formatter<'_>, shown: Shown) -> fmt::Result {
        match self {
            Self::RemoveDatabaseLock => write!(out, "rm /{DATABASE_LOCK}"),
            Self::InstallMirrorlist(mirrorlist) => {
                let servers = mirrorlist.servers();
                let plural = if servers == 1 { "" } else { "s" };

                write!(
                    out,
                    "write /{CHAOTIC_MIRRORLIST} ({servers} server{plural}), the old one kept as chaotic-mirrorlist.bak"
                )
            }
            Self::RequireListedShell(shell) => {
                write!(out, "check that {shell} is listed in /{SHELLS}")
            }
            Self::WriteDnsConfig(provider) => dns_config(out, *provider),
            Self::WriteWifiBackend(backend) => wifi_backend(out, *backend),
            Self::SetGarudaUpdateFlag { flag, on } => {
                let name = flag.key();

                if *on {
                    write!(out, "set {name}=1 in /{GARUDA_UPDATE_CONFIG}")
                } else {
                    write!(out, "comment out {name} in /{GARUDA_UPDATE_CONFIG}")
                }
            }
            Self::EditUserConfig { edit, on } => {
                out.write_str(&edit.describe(Toggle::from_on(*on)))
            }
            Self::EditLocaleGen { entry, on } => {
                let verb = if *on { "enable" } else { "comment out" };
                write!(out, "{verb} `{entry}` in /{LOCALE_GEN}")
            }
            Self::RequireOrdinaryUser(user) => ordinary_user_check(out, user),
            Self::RequireFreeHome(_, home) => write!(out, "check that {home} does not exist yet"),
            Self::RequireSafeHome(home) => write!(out, "check the folders above {home}"),
            Self::RequireUserGroup(group) => {
                write!(out, "check that {group} is not a system group")
            }
            Self::WriteConfigFile(write) => out.write_str(&write.command()),
            Self::ResolveMerge(resolve) => out.write_str(&resolve.command()),
            Self::SnapshotBeforeMerges => write!(
                out,
                "{}, where snapper has a configuration of /",
                crate::merge::snapshot()
            ),
            Self::WriteHomeFile(write) => out.write_str(&write.command()),
            Self::InstallBuilt {
                run,
                packages,
                reason,
            } => {
                let names: Vec<&str> = packages.iter().map(PackageName::as_str).collect();
                let flag = reason
                    .pacman_flag()
                    .map_or_else(String::new, |flag| format!("{flag} "));

                write!(
                    out,
                    "copy {} from ~/{AUR_BUILT}/{run} to /{AUR_STORE}, then pacman -U {flag}--noconfirm --",
                    names.join(" "),
                )
            }
            Self::BuildAur { base, run } => {
                write!(
                    out,
                    "PKGDEST=~/{AUR_BUILT}/{run} makepkg, in ~/{PARU_CLONE}/{base}"
                )
            }
            Self::EditModprobeBlacklist { module, on } => {
                let verb = if *on { "add" } else { "remove" };
                write!(out, "{verb} `blacklist {module}` in /{MODPROBE_BLACKLIST}")
            }
            Self::EditGrubDefaults(settings) => write!(out, "set {settings} in /{GRUB_DEFAULTS}"),
            Self::EditLoaderConf(change) => loader_conf(out, change),
            Self::WriteKernelCmdline(parameters) => kernel_cmdline(out, parameters),
            Self::PrepareRepairMount => write!(out, "mkdir -p /{REPAIR_ROOT}"),
            Self::RequireInstalledSystem(need) => installed_system(out, *need),
            Self::RecheckRepairDevices { system: _, work: _ } => {
                out.write_str("check the devices of the repair again")
            }
            Self::SetSystemLocale(locale) => write!(out, "set LANG={locale} in /{LOCALE_CONF}"),
            Self::RebuildInitramfs { root } => initramfs::describe(out, *root),
            Self::RebuildUkis { root } => uki::describe(out, *root),
            Self::RepairBootLoader { system, work } => {
                crate::action::describe_loader(out, system, work, shown)
            }
        }
    }
}

fn dns_config(out: &mut fmt::Formatter<'_>, provider: DnsProvider) -> fmt::Result {
    if provider.config().is_some() {
        write!(
            out,
            "write /{NETWORKMANAGER_CONF_D}/{DNS_CONFIG} (servers={})",
            provider.servers().join(",")
        )
    } else {
        write!(out, "rm -f /{NETWORKMANAGER_CONF_D}/{DNS_CONFIG}")
    }
}

fn wifi_backend(out: &mut fmt::Formatter<'_>, backend: WifiBackend) -> fmt::Result {
    if backend.config().is_some() {
        write!(
            out,
            "write /{NETWORKMANAGER_CONF_D}/{WIFI_BACKEND_CONFIG} (wifi.backend={})",
            backend.as_str()
        )
    } else {
        write!(
            out,
            "rm -f /{NETWORKMANAGER_CONF_D}/{WIFI_BACKEND_CONFIG} /{NETWORKMANAGER_CONF_D}/{OLD_WIFI_BACKEND_CONFIG}"
        )
    }
}

fn installed_system(out: &mut fmt::Formatter<'_>, need: RepairNeed) -> fmt::Result {
    const SDBOOT_ESP: &str = "its EFI system partition in /etc/fstab";

    let programs = match need {
        RepairNeed::Menu => "grub-mkconfig, or on systemd-boot kernel-install",
        RepairNeed::MenuWithEsp => {
            &format!("grub-mkconfig, or on systemd-boot kernel-install and {SDBOOT_ESP}")
        }
        RepairNeed::Installer => "grub-install and grub-mkconfig",
        RepairNeed::InstallerWithEsp => &format!(
            "grub-install, grub-mkconfig and the folder /{ESP_FOLDER}, or on systemd-boot bootctl, kernel-install and {SDBOOT_ESP}"
        ),
    };

    write!(
        out,
        "check that /{REPAIR_ROOT} holds an installed system with a kernel, {programs}"
    )
}

fn loader_conf(out: &mut fmt::Formatter<'_>, change: &LoaderChange) -> fmt::Result {
    write!(
        out,
        "set {change} in {LOADER_CONF} on the EFI system partition"
    )
}

fn kernel_cmdline(out: &mut fmt::Formatter<'_>, parameters: &KernelParameters) -> fmt::Result {
    write!(out, "write {parameters} to /{KERNEL_CMDLINE}")
}

fn ordinary_user_check(out: &mut fmt::Formatter<'_>, user: &UserName) -> fmt::Result {
    write!(
        out,
        "check that {user} is an ordinary account (id {FIRST_USER_ID} to {})",
        crate::LAST_USER_ID
    )
}
