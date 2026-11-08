#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use super::{TargetList, UnitOperation, UnitScope, network, on_units, run};
use crate::procedure::{Builtin, Command, Invocation, Program};
use crate::{
    ConfigReload, EditCopy, Editor, HomeWrite, HopLimit, HostName, PackageName, PingCount,
    PingTimeout, UnitName, UserConfigEdit,
};

/// Enough history to see why a unit failed without flooding the Console.
const JOURNAL_LINES: &str = "500";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum UserAction {
    /// paru must never run as root. It asks for root itself through pkexec.
    /// Without `--noconfirm`: the user reads the PKGBUILD diffs and answers
    /// paru in the Console, which is the point of running it there.
    UpgradeAur,
    /// Builds and installs AUR packages. As the upgrade: as the user,
    /// without `--noconfirm`, so paru shows each build script in the
    /// Console before it builds.
    InstallAur {
        packages: TargetList<PackageName>,
    },
    EnableUserUnits {
        units: TargetList<UnitName>,
    },
    DisableUserUnits {
        units: TargetList<UnitName>,
    },
    ControlUserUnits {
        operation: UnitOperation,
        units: TargetList<UnitName>,
    },
    MaskUserUnits {
        units: TargetList<UnitName>,
    },
    UnmaskUserUnits {
        units: TargetList<UnitName>,
    },
    /// The journal of one unit. Runs as the user for system units too:
    /// reading the system journal needs `wheel`, `adm` or
    /// `systemd-journal`, never root.
    UnitLogs {
        unit: UnitName,
        scope: UnitScope,
    },
    /// The read-only health check. Its fixes need root
    /// ([`crate::RootAction::GarudaHealthFix`]).
    GarudaHealth,
    /// The app writes it itself, no program runs.
    EditUserConfig {
        edit: UserConfigEdit,
        on: bool,
    },
    ReloadConfig {
        target: ConfigReload,
    },
    /// A terminal editor on the user's private copy of a config file, in
    /// the Console. The editor never runs as root and never opens the
    /// file itself: [`crate::RootAction::WriteConfigFile`] carries the
    /// result back.
    EditFile {
        editor: Editor,
        copy: EditCopy,
    },
    WriteHomeFile {
        write: HomeWrite,
    },
    /// A connection test in the Console. ping needs no root: the kernel
    /// lets every user send echo requests.
    Ping {
        host: HostName,
        count: PingCount,
        timeout: PingTimeout,
    },
    Traceroute {
        host: HostName,
        max_hops: HopLimit,
    },
}

impl UserAction {
    pub fn procedure(&self) -> Vec<Invocation> {
        match self {
            Self::UpgradeAur => vec![run(
                Command::new(Program::Paru).args(["--sudo", "pkexec", "-Sua"])
            )],
            Self::InstallAur { packages } => vec![run(Command::new(Program::Paru)
                .args(["--sudo", "pkexec", "-S", "--aur", "--"])
                .args(packages.iter().map(|package| package.as_str().to_owned())))],
            Self::EnableUserUnits { units } => on_units(["--user", "enable", "--now"], units),
            Self::DisableUserUnits { units } => on_units(["--user", "disable", "--now"], units),
            Self::ControlUserUnits { operation, units } => {
                on_units(["--user", operation.as_str()], units)
            }
            Self::MaskUserUnits { units } => on_units(["--user", "mask"], units),
            Self::UnmaskUserUnits { units } => on_units(["--user", "unmask"], units),
            // `-u` takes the unit as its argument, so no `--` is needed,
            // and unit names never start with `-` anyway.
            Self::UnitLogs { unit, scope } => vec![run(journal(*scope)
                .args(["--no-pager", "-n", JOURNAL_LINES, "-u"])
                .arg(unit.as_str()))],
            Self::GarudaHealth => vec![run(Command::new(Program::GarudaHealth))],
            Self::ReloadConfig { target } => vec![run(target.command())],
            Self::EditFile { editor, copy } => vec![run(editor.command(copy))],
            Self::WriteHomeFile { write } => vec![write.invocation()],
            Self::Ping {
                host,
                count,
                timeout,
            } => network::ping(host, *count, *timeout),
            Self::Traceroute { host, max_hops } => network::traceroute(host, *max_hops),
            Self::EditUserConfig { edit, on } => {
                vec![Invocation::Builtin(Builtin::EditUserConfig {
                    edit: edit.clone(),
                    on: *on,
                })]
            }
        }
    }
}

fn journal(scope: UnitScope) -> Command {
    let command = Command::new(Program::Journalctl);

    match scope {
        UnitScope::System => command,
        UnitScope::User => command.arg("--user"),
    }
}
