//! Program checks take an `exists` function, so tests never depend on what is installed.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::path::Path;

use toolbox_plan::{PackageName, Plan, RemoteAction, RootAction, TargetList, Upgrader};

use super::{
    GARUDA_HEALTH_PACKAGE, GARUDA_UPDATE_PACKAGE, Operation, Operations, Task, Tool,
    plan_aur_upgrade, plan_health_check, plan_health_fix, plan_now, plan_remote, upgrader_for,
};
use crate::domains::InstalledPackages;
use crate::features::live::plan_chroot;
use crate::{ChangeSet, Snapshot};

pub const GARUDA_UPDATE_PROGRAM: &str = "/usr/bin/garuda-update";
const GARUDA_HEALTH: &str = "/usr/bin/garuda-health";
/// Where Garuda installs `rate-mirrors`, which ranks the Chaotic-AUR
/// mirrors.
pub const RATE_MIRRORS_PROGRAM: &str = "/usr/bin/rate-mirrors";
pub const CHAOTIC_MIRRORS: &str = "chaotic-mirrors";
/// The operation the Home page offers too.
pub const UPGRADE_SYSTEM: &str = Operation::UpgradeSystem.key();

/// The AUR update, run now only, as paru asks its own questions.
pub const AUR_UPGRADE: &str = "aur-upgrade";

const CHROOT: &str = "chroot";

/// The operations in the order the page lists them, the update first.
const OPERATIONS: [Operation; 5] = [
    Operation::UpgradeSystem,
    Operation::RemoveOrphans,
    Operation::CleanPackageCache,
    Operation::RefreshKeyring,
    Operation::RemoveDatabaseLock,
];

/// The keys of [`OPERATIONS`], which the command line takes.
pub const OPERATION_KEYS: [&str; 5] = [
    UPGRADE_SYSTEM,
    Operation::RemoveOrphans.key(),
    Operation::CleanPackageCache.key(),
    Operation::RefreshKeyring.key(),
    Operation::RemoveDatabaseLock.key(),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationRow {
    pub operation: Operation,
    pub queued: bool,
}

/// One tool the page opens, or installs first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolRow {
    pub tool: PageTool,
    pub installed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepairRow {
    pub repair: Repair,
    pub available: bool,
    /// The package that brings its program, named while it is missing.
    pub needs: PackageName,
}

/// A task of the Repairs tab, which runs at once in the Console.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repair {
    HealthCheck,
    HealthFix,
    Remote(RemoteAction),
}

/// A row of the tools card: a program the page starts, or the ranking of
/// the Chaotic-AUR mirrors, which the app runs itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageTool {
    Program(Tool),
    ChaoticMirrors,
}

/// Why a launched program is missing, in the words a user can act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchCause {
    /// Its install ended well, yet the program is not where it should be,
    /// so the system has no error of its own to show.
    AbsentAfterInstall,
}

impl Repair {
    /// In the order the page lists them.
    pub const ALL: [Self; 8] = [
        Self::HealthCheck,
        Self::HealthFix,
        Self::Remote(RemoteAction::Fix),
        Self::Remote(RemoteAction::Keyring),
        Self::Remote(RemoteAction::FullFix),
        Self::Remote(RemoteAction::ResetAudio),
        Self::Remote(RemoteAction::ResetSnapper),
        Self::Remote(RemoteAction::Reinstall),
    ];

    /// The page's id. A remote action's (`remote-full-fix`) differs from
    /// its garuda-update argument (`fullfix`).
    pub const fn key(self) -> &'static str {
        match self {
            Self::HealthCheck => "health-check",
            Self::HealthFix => "health-fix",
            Self::Remote(RemoteAction::Fix) => "remote-fix",
            Self::Remote(RemoteAction::Keyring) => "remote-keyring",
            Self::Remote(RemoteAction::FullFix) => "remote-full-fix",
            Self::Remote(RemoteAction::ResetAudio) => "remote-reset-audio",
            Self::Remote(RemoteAction::ResetSnapper) => "remote-reset-snapper",
            Self::Remote(RemoteAction::Reinstall) => "remote-reinstall",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|repair| repair.key() == key)
    }

    /// The id of its name on the Repairs tab.
    pub fn text_id(self) -> String {
        format!("maintenance-repair-{}", self.key())
    }

    /// It reinstalls packages or throws away what the user had
    /// (snapshots, `pacman.conf`, the audio setup), which no later click
    /// brings back.
    pub const fn asks_first(self) -> bool {
        match self {
            Self::Remote(
                RemoteAction::Fix
                | RemoteAction::FullFix
                | RemoteAction::ResetAudio
                | RemoteAction::ResetSnapper
                | RemoteAction::Reinstall,
            ) => true,
            Self::HealthCheck | Self::HealthFix | Self::Remote(RemoteAction::Keyring) => false,
        }
    }

    #[must_use = "a plan does nothing until it is run"]
    pub fn plan(self) -> Plan {
        match self {
            Self::HealthCheck => plan_health_check(),
            Self::HealthFix => plan_health_fix(),
            Self::Remote(action) => plan_remote(action),
        }
    }
}

impl PageTool {
    pub fn key(self) -> &'static str {
        match self {
            Self::Program(tool) => tool.id(),
            Self::ChaoticMirrors => CHAOTIC_MIRRORS,
        }
    }

    /// The id of its name on the Common tab.
    pub fn text_id(self) -> String {
        format!("maintenance-tool-{}", self.key())
    }
}

/// The upgrader the system has: garuda-update when installed. Without the
/// installed packages, `program_found` says whether its program was there.
pub fn upgrader(snapshot: &Snapshot, program_found: bool) -> Upgrader {
    snapshot.part::<InstalledPackages>().map_or(
        if program_found {
            Upgrader::GarudaUpdate
        } else {
            Upgrader::Pacman
        },
        upgrader_for,
    )
}

/// The operation the command line names `id`.
pub fn task(id: &str, upgrader: Upgrader) -> Option<Task> {
    OPERATIONS
        .into_iter()
        .find(|operation| operation.key() == id)
        .map(|operation| operation.task(upgrader))
}

/// The run of the task the command line names `id`.
pub fn run_now_plan(id: &str, upgrader: Upgrader) -> Option<Plan> {
    if let Some(task) = task(id, upgrader) {
        return Some(plan_now(task));
    }

    match id {
        AUR_UPGRADE => Some(plan_aur_upgrade()),
        CHROOT => Some(plan_chroot()),
        other => Repair::from_key(other).map(Repair::plan),
    }
}

pub fn operations(changes: &ChangeSet) -> Vec<OperationRow> {
    let queued: BTreeSet<Operation> = changes
        .queued::<Operations>()
        .iter()
        .map(|(operation, _)| *operation)
        .collect();

    OPERATIONS
        .into_iter()
        .map(|operation| OperationRow {
            operation,
            queued: queued.contains(&operation),
        })
        .collect()
}

pub fn tools<F>(exists: F) -> Vec<ToolRow>
where
    F: Fn(&Path) -> bool,
{
    Tool::ALL
        .iter()
        .flat_map(|tool| {
            let row = ToolRow {
                tool: PageTool::Program(*tool),
                installed: exists(Path::new(tool.program())),
            };

            let ranking = matches!(tool, Tool::ArchMirrors).then(|| ToolRow {
                tool: PageTool::ChaoticMirrors,
                installed: exists(Path::new(RATE_MIRRORS_PROGRAM)),
            });

            std::iter::once(row).chain(ranking)
        })
        .collect()
}

pub fn repairs<F>(exists: F) -> Vec<RepairRow>
where
    F: Fn(&Path) -> bool,
{
    let health = exists(Path::new(GARUDA_HEALTH));
    let update = exists(Path::new(GARUDA_UPDATE_PROGRAM));

    Repair::ALL
        .into_iter()
        .map(|repair| match repair {
            Repair::HealthCheck | Repair::HealthFix => RepairRow {
                repair,
                available: health,
                needs: GARUDA_HEALTH_PACKAGE.clone(),
            },
            Repair::Remote(_) => RepairRow {
                repair,
                available: update,
                needs: GARUDA_UPDATE_PACKAGE.clone(),
            },
        })
        .collect()
}

/// Whether the task the command line names `id` asks before it runs.
pub fn asks_first(id: &str) -> bool {
    OPERATIONS
        .into_iter()
        .find(|operation| operation.key() == id)
        .map(Operation::asks_first)
        .or_else(|| Repair::from_key(id).map(Repair::asks_first))
        .unwrap_or(false)
}

/// The packages the repairs of `rows` miss, each once, in the order the
/// rows name them.
pub fn missing(rows: &[RepairRow]) -> Vec<&PackageName> {
    let mut missing: Vec<&PackageName> = Vec::new();
    for row in rows.iter().filter(|row| !row.available) {
        if !missing.contains(&&row.needs) {
            missing.push(&row.needs);
        }
    }

    missing
}

/// One install of every package the repairs of `rows` miss, `None` while
/// none is missing.
pub fn rescue_install(rows: &[RepairRow]) -> Option<Plan> {
    let packages = TargetList::new(missing(rows).into_iter().cloned()).ok()?;
    Some(Plan::root(RootAction::InstallPackages { packages }))
}

pub fn install(package: &PackageName) -> Plan {
    Plan::root(crate::domains::install(package))
}

pub fn installed(rows: &[ToolRow], tool: PageTool) -> bool {
    rows.iter().any(|row| row.tool == tool && row.installed)
}
