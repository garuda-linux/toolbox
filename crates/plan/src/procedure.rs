//! The exact programs an action runs. The helper executes these, and the
//! Review page shows them, so both always agree.

mod program;
#[cfg(test)]
mod tests;

use std::fmt;

pub use program::Program;

use crate::garuda_update::GARUDA_UPDATE_CONFIG;
use crate::grub::GRUB_DEFAULTS;
use crate::locale::{LOCALE_CONF, LOCALE_GEN};
use crate::network::MODPROBE_BLACKLIST;
use crate::repair::{ESP_FOLDER, REPAIR_ROOT, RepairNeed};
use crate::settings::{
    DNS_CONFIG, NETWORKMANAGER_CONF_D, OLD_WIFI_BACKEND_CONFIG, WIFI_BACKEND_CONFIG,
};
use crate::{
    ConfigWrite, DnsProvider, GarudaUpdateFlag, GroupName, GrubSettings, HomeDirectory, HomeWrite,
    InstalledSystem, KernelModule, LocaleEntry, LocaleName, Mirrorlist, RepairWork, SecretInput,
    ShellPath, UserConfigEdit, UserName, WifiBackend,
};

/// Where [`Builtin::InstallMirrorlist`] writes, relative to `/`.
pub const CHAOTIC_MIRRORLIST: &str = "etc/pacman.d/chaotic-mirrorlist";

/// The lock pacman holds while it runs, relative to `/`.
pub const DATABASE_LOCK: &str = "var/lib/pacman/db.lck";

/// The login shells the system allows, relative to `/`.
pub const SHELLS: &str = "etc/shells";

/// The account files the account guards read, relative to `/`.
pub const PASSWD: &str = "etc/passwd";
pub const GROUP: &str = "etc/group";

/// `UID_MIN` and `GID_MIN` of `/etc/login.defs`: ids below belong to the
/// system.
pub const FIRST_USER_ID: u32 = 1000;

/// One program run: argv plus the variables it gets on top of the
/// helper's clean environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    program: Program,
    args: Vec<String>,
    env: Vec<(&'static str, &'static str)>,
}

impl Command {
    #[must_use = "a command does nothing until it is run"]
    pub const fn new(program: Program) -> Self {
        Self {
            program,
            args: Vec::new(),
            env: Vec::new(),
        }
    }

    #[must_use = "builder methods return the changed command"]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    #[must_use = "builder methods return the changed command"]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    #[must_use = "builder methods return the changed command"]
    pub fn env(mut self, key: &'static str, value: &'static str) -> Self {
        self.env.push((key, value));
        self
    }

    pub const fn program(&self) -> Program {
        self.program
    }

    pub fn arguments(&self) -> &[String] {
        &self.args
    }

    pub fn variables(&self) -> &[(&'static str, &'static str)] {
        &self.env
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in &self.env {
            write!(f, "{key}={value} ")?;
        }

        f.write_str(self.program.name())?;
        for arg in &self.args {
            write!(f, " {}", quoted(arg))?;
        }

        Ok(())
    }
}

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
    /// Fails for the account with user id 0 (whatever it is called) and
    /// for an account `/etc/passwd` does not list. The app's own guard is
    /// no protection once a request is on the wire.
    RequireOrdinaryUser(UserName),
    /// Fails when the folder exists. `usermod -d <new> -m` writes the new
    /// home into `/etc/passwd` before it refuses a folder that exists, so a
    /// move onto one would leave the account pointing at a foreign folder.
    RequireFreeHome(HomeDirectory),
    /// Fails for a group with an id below [`FIRST_USER_ID`]: packages own
    /// those and files on disk refer to them.
    RequireUserGroup(GroupName),
    /// Replaces a config file with the text the user edited, keeping
    /// owner and mode and the previous version in the backup folder. Fails
    /// when the file no longer is the version the edit started from.
    WriteConfigFile(ConfigWrite),
    /// [`Builtin::WriteConfigFile`] for a file in the user's home. Runs only
    /// in the app, as the user: the helper has no home to write.
    WriteHomeFile(HomeWrite),
    /// Adds a module to the app's own modprobe file or takes it out again.
    /// No other modprobe file is ever written.
    EditModprobeBlacklist { module: KernelModule, on: bool },
    /// Sets variables of `/etc/default/grub` and keeps the old file
    /// beside it. Fails without the file: GRUB is not the boot loader then.
    EditGrubDefaults(GrubSettings),
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
}

impl fmt::Display for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RemoveDatabaseLock => {
                write!(f, "rm /{DATABASE_LOCK}  # only while pacman is not running")
            }
            Self::InstallMirrorlist(mirrorlist) => {
                let servers = mirrorlist.servers();
                let plural = if servers == 1 { "" } else { "s" };

                write!(
                    f,
                    "write /{CHAOTIC_MIRRORLIST} ({servers} server{plural}), the old one kept as chaotic-mirrorlist.bak"
                )
            }
            Self::RequireListedShell(shell) => {
                write!(f, "check that {shell} is listed in /{SHELLS}")
            }
            Self::WriteDnsConfig(provider) => dns_config(f, *provider),
            Self::WriteWifiBackend(backend) => wifi_backend(f, *backend),
            Self::SetGarudaUpdateFlag { flag, on } => {
                let name = flag.key();

                if *on {
                    write!(f, "set {name}=1 in /{GARUDA_UPDATE_CONFIG}")
                } else {
                    write!(f, "comment out {name} in /{GARUDA_UPDATE_CONFIG}")
                }
            }
            Self::EditUserConfig { edit, on } => f.write_str(&edit.describe(*on)),
            Self::EditLocaleGen { entry, on } => {
                let verb = if *on { "enable" } else { "comment out" };

                write!(f, "{verb} `{entry}` in /{LOCALE_GEN}")
            }
            Self::RequireOrdinaryUser(user) => {
                write!(f, "check that {user} is not the root account")
            }
            Self::RequireFreeHome(home) => write!(f, "check that {home} does not exist yet"),
            Self::RequireUserGroup(group) => write!(f, "check that {group} is not a system group"),
            Self::WriteConfigFile(write) => write.fmt(f),
            Self::WriteHomeFile(write) => write.fmt(f),
            Self::EditModprobeBlacklist { module, on } => {
                let verb = if *on { "add" } else { "remove" };

                write!(f, "{verb} `blacklist {module}` in /{MODPROBE_BLACKLIST}")
            }
            Self::EditGrubDefaults(settings) => write!(f, "set {settings} in /{GRUB_DEFAULTS}"),
            Self::PrepareRepairMount => write!(
                f,
                "mkdir -p /{REPAIR_ROOT}  # for root alone, nothing may be mounted there"
            ),
            Self::RequireInstalledSystem(need) => installed_system(f, *need),
            Self::RecheckRepairDevices { system: _, work: _ } => {
                f.write_str("check the devices of the repair again")
            }
            Self::SetSystemLocale(locale) => write!(
                f,
                "set LANG={locale} in /{LOCALE_CONF}  # only when /{LOCALE_GEN} has it enabled"
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invocation {
    Run(Command),
    /// Runs `query`, then `run` with every line of the query's output as an
    /// extra argument. Skipped when the query prints nothing.
    RunWithTargetsFrom {
        query: Command,
        run: Command,
    },
    Builtin(Builtin),
    /// Runs even when an earlier invocation of the same step failed or was
    /// cancelled, and cannot be cancelled itself. For work that must undo a
    /// temporary state, such as starting a service the step stopped. The
    /// step still reports the earlier failure.
    Always(Command),
    /// Runs `command` without a terminal and writes `input` to its
    /// standard input: how a secret reaches a program. `None` while the
    /// secret has not arrived, which fails the step.
    Feed {
        command: Command,
        input: Option<SecretInput>,
    },
}

impl fmt::Display for Invocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Run(command) => command.fmt(f),
            Self::RunWithTargetsFrom { query, run } => write!(f, "{run} $({query})"),
            Self::Builtin(builtin) => builtin.fmt(f),
            Self::Always(command) => write!(f, "{command}  # always, even after a failure"),
            Self::Feed { command, input: _ } => {
                write!(f, "{command}  # the secret arrives on its standard input")
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
    let programs = match need {
        RepairNeed::Menu => "grub-mkconfig",
        RepairNeed::Installer => "grub-install and grub-mkconfig",
        RepairNeed::InstallerWithEsp => {
            &format!("grub-install, grub-mkconfig and the folder /{ESP_FOLDER}")
        }
    };

    write!(
        out,
        "check that /{REPAIR_ROOT} holds an installed system with a kernel, {programs}"
    )
}

/// Single quotes for display when an argument holds anything a shell would
/// treat specially. Validated names never need it.
fn quoted(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_./=:@+,".contains(character));

    if plain {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}
