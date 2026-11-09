//! How the root helper starts: through pkexec normally, or directly in
//! dry-run mode for the self-test (validates and reports, runs nothing).

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use tokio::process::Command;
use toolbox_plan::ConfigFilePath;
use toolbox_plan::protocol::{Mode, ReadKind};
use toolbox_system::{HelperFetch, HelperReads};

use super::Inhibitor;

/// The installed helper. Off `PATH`, and the polkit actions pin it.
const INSTALLED_HELPER: &str = "/usr/lib/garuda-toolbox/garuda-toolbox-helper";
const HELPER_NAME: &str = "garuda-toolbox-helper";
pub const PKEXEC: &str = "/usr/bin/pkexec";
pub const SUDO: &str = "/usr/bin/sudo";

/// The name `PATH` is searched for, which the doctor looks up the same way.
pub const SUDO_NAME: &str = "sudo";

/// The helper's first argument, which the apply polkit action is bound to.
const APPLY: &str = "apply";
const FETCH: &str = "fetch";

pub trait Launcher: Send + Sync {
    fn command(&self) -> Command;

    /// Dry runs reach user steps too: the self-test never changes the
    /// machine.
    fn mode(&self) -> Mode;

    /// The way to try once more when pkexec cannot ask at all. Only the
    /// command line has one, the window has no terminal for sudo.
    fn fallback(&self) -> Option<std::sync::Arc<dyn Launcher>> {
        None
    }

    /// Only the real helper changes the system, so only it holds shutdown
    /// back. Fakes in tests leave logind alone.
    fn inhibitor(&self) -> Inhibitor {
        Inhibitor::Off
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelperLauncher {
    Pkexec {
        helper: PathBuf,
        pkexec: PathBuf,
    },
    /// The command line's way when pkexec cannot ask: sudo asks in the
    /// terminal. The helper still checks every plan it gets.
    Sudo {
        helper: PathBuf,
        sudo: PathBuf,
    },
    /// Without root, for the self-test.
    DryRun {
        helper: PathBuf,
    },
}

impl HelperLauncher {
    pub fn from_process() -> Self {
        Self::from_lookup(|key| std::env::var_os(key))
    }

    /// `GARUDA_TOOLBOX_HELPER=dry-run` selects the dry run. Development
    /// builds use the helper built next to the app.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<OsString>) -> Self {
        let helper = helper_path();

        if lookup("GARUDA_TOOLBOX_HELPER").is_some_and(|value| value == "dry-run") {
            Self::DryRun { helper }
        } else {
            let path = lookup("PATH");
            let pkexec = root_tool(
                "pkexec",
                PKEXEC,
                path.as_deref(),
                cfg!(debug_assertions),
                is_setuid_root,
            );

            Self::Pkexec { helper, pkexec }
        }
    }

    /// The same helper through sudo, for the command line on a terminal.
    /// `None` in a dry run and without a sudo that can raise.
    pub fn through_sudo(&self) -> Option<Self> {
        let path = std::env::var_os("PATH");
        let sudo = root_tool(
            SUDO_NAME,
            SUDO,
            path.as_deref(),
            cfg!(debug_assertions),
            is_setuid_root,
        );

        is_setuid_root(&sudo)
            .then(|| self.with_sudo(sudo))
            .flatten()
    }

    fn with_sudo(&self, sudo: PathBuf) -> Option<Self> {
        match self {
            Self::Pkexec { helper, pkexec: _ } | Self::Sudo { helper, sudo: _ } => {
                Some(Self::Sudo {
                    helper: helper.clone(),
                    sudo,
                })
            }
            Self::DryRun { helper: _ } => None,
        }
    }

    /// Whether pkexec can raise at all: setuid root. Checked before a run,
    /// so a broken one is never tried first.
    pub fn pkexec_usable(&self) -> bool {
        match self {
            Self::Pkexec { helper: _, pkexec } => is_setuid_root(pkexec),
            Self::Sudo { helper: _, sudo: _ } | Self::DryRun { helper: _ } => true,
        }
    }

    /// The helper with `arguments`, through pkexec or sudo unless in a dry
    /// run.
    fn helper(&self, arguments: &[&str]) -> std::process::Command {
        let mut command = match self {
            Self::Pkexec { helper, pkexec } => {
                let mut command = std::process::Command::new(pkexec);
                command.arg(helper);
                command
            }
            Self::Sudo { helper, sudo } => {
                let mut command = std::process::Command::new(sudo);
                command.arg("--").arg(helper);
                command
            }
            Self::DryRun { helper } => std::process::Command::new(helper),
        };

        command.args(arguments);
        command
    }
}

impl Launcher for HelperLauncher {
    fn command(&self) -> Command {
        self.helper(&[APPLY]).into()
    }

    fn mode(&self) -> Mode {
        match self {
            Self::Pkexec {
                helper: _,
                pkexec: _,
            }
            | Self::Sudo { helper: _, sudo: _ } => Mode::Run,
            Self::DryRun { helper: _ } => Mode::DryRun,
        }
    }

    fn inhibitor(&self) -> Inhibitor {
        Inhibitor::Logind
    }
}

/// The Diagnostics page's reads of logs the user may not read.
impl HelperReads for HelperLauncher {
    fn read_command(&self, kind: ReadKind) -> std::process::Command {
        self.helper(&[kind.command(), kind.as_str()])
    }
}

/// The Config Files page's reads of files the user may not read.
impl HelperFetch for HelperLauncher {
    fn fetch_command(&self, path: &ConfigFilePath) -> std::process::Command {
        self.helper(&[FETCH, path.as_str()])
    }
}

/// A root tool (`pkexec`, `sudo`) at its Arch place `fixed`.
///
/// A development build also looks on `PATH`, as NixOS keeps them in
/// `/run/wrappers/bin`, but only takes a setuid root copy: a dev shell may
/// put the Nix store's plain one first. A release never searches `PATH`
/// for the program it hands root to.
pub fn root_tool(
    name: &str,
    fixed: &str,
    path: Option<&std::ffi::OsStr>,
    development: bool,
    setuid_root: impl Fn(&Path) -> bool,
) -> PathBuf {
    let on_path = path
        .filter(|_| development)
        .into_iter()
        .flat_map(std::env::split_paths)
        .map(|dir| dir.join(name))
        .find(|candidate| setuid_root(candidate));

    on_path.unwrap_or_else(|| PathBuf::from(fixed))
}

/// Only a root-owned setuid program can raise its child to root.
pub fn is_setuid_root(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt as _;

    const SETUID: u32 = 0o4000;
    std::fs::metadata(path)
        .is_ok_and(|meta| meta.is_file() && meta.uid() == 0 && meta.mode() & SETUID != 0)
}

/// The installed helper, or in a debug build the one built beside the
/// app (polkit then uses pkexec's generic action and asks every time).
pub fn helper_path() -> PathBuf {
    let beside_app = std::env::current_exe()
        .ok()
        .and_then(|app| app.parent().map(|dir| dir.join(HELPER_NAME)))
        .filter(|helper| helper.is_file());

    match beside_app {
        Some(helper) if cfg!(debug_assertions) => helper,
        Some(_) | None => PathBuf::from(INSTALLED_HELPER),
    }
}
