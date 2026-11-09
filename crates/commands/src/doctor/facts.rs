//! What `doctor` looks at, read from the running system. Every read here
//! only looks: no file is created or changed, no root tool runs.

use std::ffi::OsStr;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};

use tokio::runtime::Runtime;
use toolbox_config::Paths;
use toolbox_core::features::maintenance::page::GARUDA_UPDATE_PROGRAM;
use toolbox_plan::UnitScope;
use toolbox_services::apply::launcher::{
    PKEXEC, SUDO, SUDO_NAME, helper_path, is_setuid_root, root_tool,
};
use toolbox_services::catalogs::{Catalogs, catalog_dir};
use toolbox_services::log_folder::log_dir;
use toolbox_system::{CallError, Method, call, connect};

/// The policy's file name, as `scripts/install.sh` installs it.
const POLICY: &str = "org.garudalinux.toolbox.policy";

/// Where polkit reads actions: Arch's place, the admin's and NixOS's.
const POLICY_DIRS: [&str; 3] = [
    "/usr/share/polkit-1/actions",
    "/etc/polkit-1/actions",
    "/run/current-system/sw/share/polkit-1/actions",
];

const PACMAN_LOCAL: &str = "/var/lib/pacman/local";
const PACMAN_SYNC: &str = "/var/lib/pacman/sync";

/// The log file's name, as `main.rs` opens it.
const LOG_FILE: &str = "garuda-toolbox.log";

/// A ping reaches the manager without asking it anything.
const PING: Method = Method {
    destination: "org.freedesktop.systemd1",
    path: "/org/freedesktop/systemd1",
    interface: "org.freedesktop.DBus.Peer",
    name: "Ping",
};

/// The names of known polkit agents that have neither `polkit` nor
/// `policykit` in their name. A desktop shell may carry its own.
const AGENT_SHELLS: [&str; 4] = ["gnome-shell", "cinnamon", "cosmic-osd", "soteria"];

/// polkit's daemon and the setuid helper an agent starts, which are no
/// agents themselves.
const NO_AGENTS: [&str; 2] = ["polkitd", "polkit-agent-he"];

pub const PROGRAMS: [&str; 5] = [
    "checkupdates",
    "paru",
    "garuda-update",
    "garuda-health",
    "snapper",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tool {
    pub path: PathBuf,
    pub present: bool,
    pub setuid_root: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pacman {
    pub local: bool,
    pub repositories: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramFacts {
    pub installed: Vec<&'static str>,
}

impl ProgramFacts {
    pub fn has(&self, name: &str) -> bool {
        self.installed.contains(&name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogFacts {
    pub dir: PathBuf,
    /// A release refuses to open its window without the folder.
    pub present: bool,
    pub loaded: usize,
    pub problems: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogFacts {
    /// `None` when no safe folder was found.
    pub file: Option<PathBuf>,
    pub writable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub development: bool,
    /// `GARUDA_TOOLBOX_HELPER=dry-run`, the self-test's mode.
    pub dry_run: bool,
    pub pkexec: Tool,
    pub sudo: Tool,
    pub agent: Option<String>,
    pub helper: PathBuf,
    pub helper_present: bool,
    pub policy: Option<PathBuf>,
    pub pacman: Pacman,
    pub programs: ProgramFacts,
    pub system_bus: Result<(), CallError>,
    pub user_bus: Result<(), CallError>,
    pub catalogs: CatalogFacts,
    pub log: LogFacts,
}

pub fn agent(names: &[String]) -> Option<String> {
    names
        .iter()
        .filter(|name| !NO_AGENTS.contains(&name.as_str()))
        .find(|name| {
            name.contains("polkit")
                || name.contains("policykit")
                || AGENT_SHELLS.contains(&name.as_str())
        })
        .cloned()
}

pub fn gather(runtime: &Runtime) -> Facts {
    let development = cfg!(debug_assertions);
    let path = std::env::var_os("PATH");
    let helper = helper_path();
    let (system_bus, user_bus) =
        runtime.block_on(async { tokio::join!(ping(UnitScope::System), ping(UnitScope::User)) });

    Facts {
        development,
        dry_run: std::env::var_os("GARUDA_TOOLBOX_HELPER").is_some_and(|value| value == "dry-run"),
        pkexec: tool("pkexec", PKEXEC, path.as_deref(), development),
        sudo: tool(SUDO_NAME, SUDO, path.as_deref(), development),
        agent: agent(&own_processes()),
        helper_present: executable(&helper),
        helper,
        policy: POLICY_DIRS
            .iter()
            .map(|dir| Path::new(dir).join(POLICY))
            .find(|file| file.is_file()),
        pacman: Pacman {
            local: std::fs::read_dir(PACMAN_LOCAL).is_ok(),
            repositories: repositories(Path::new(PACMAN_SYNC)),
        },
        programs: ProgramFacts {
            installed: PROGRAMS
                .into_iter()
                .filter(|name| installed(name, path.as_deref()))
                .collect(),
        },
        system_bus,
        user_bus,
        catalogs: catalogs(),
        log: log(),
    }
}

fn tool(name: &str, fixed: &str, path: Option<&OsStr>, development: bool) -> Tool {
    let found = root_tool(name, fixed, path, development, is_setuid_root);
    Tool {
        present: found.is_file(),
        setuid_root: is_setuid_root(&found),
        path: found,
    }
}

async fn ping(scope: UnitScope) -> Result<(), CallError> {
    let connection = connect(scope).await?;
    call::<(), ()>(&connection, &PING, &()).await
}

/// The names of this user's processes. `comm` is cut to 15 bytes.
fn own_processes() -> Vec<String> {
    let me = rustix::process::getuid().as_raw();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.bytes().all(|byte| byte.is_ascii_digit()))
        })
        .filter(|entry| entry.metadata().is_ok_and(|meta| meta.uid() == me))
        .filter_map(|entry| std::fs::read_to_string(entry.path().join("comm")).ok())
        .map(|name| name.trim().to_owned())
        .collect()
}

fn executable(path: &Path) -> bool {
    std::fs::metadata(path)
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

/// garuda-update has a fixed place, as the upgrade looks there.
fn installed(name: &str, path: Option<&OsStr>) -> bool {
    if name == "garuda-update" {
        Path::new(GARUDA_UPDATE_PROGRAM).is_file()
    } else {
        on_path(name, path)
    }
}

fn on_path(name: &str, path: Option<&OsStr>) -> bool {
    path.into_iter()
        .flat_map(std::env::split_paths)
        .any(|dir| executable(&dir.join(name)))
}

fn repositories(sync: &Path) -> usize {
    std::fs::read_dir(sync).map_or(0, |entries| {
        entries
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|ending| ending == "db")
            })
            .count()
    })
}

fn catalogs() -> CatalogFacts {
    let dir = catalog_dir(|key| std::env::var_os(key));
    let loaded = Catalogs::load(&dir);
    let toggles = [
        &loaded.components,
        &loaded.gaming,
        &loaded.settings,
        &loaded.setup,
    ];

    let mut problems: Vec<String> = toggles
        .iter()
        .filter_map(|toggles| toggles.catalog.as_ref().err())
        .map(|problem| problem.detail.clone())
        .collect();

    let good = toggles.len() - problems.len() + usize::from(loaded.apps.problem.is_none());
    problems.extend(loaded.apps.problem.map(|problem| problem.detail));

    CatalogFacts {
        present: dir.is_dir(),
        dir,
        loaded: good,
        problems,
    }
}

/// The log file `main.rs` opened before the command ran. Opening it to
/// append, without creating it, writes nothing.
fn log() -> LogFacts {
    let file = log_dir(&Paths::from_process()).map(|dir| dir.join(LOG_FILE));
    let writable = file
        .as_deref()
        .is_some_and(|file| std::fs::OpenOptions::new().append(true).open(file).is_ok());

    LogFacts { file, writable }
}
