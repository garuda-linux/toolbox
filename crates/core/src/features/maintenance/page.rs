//! Program checks take an `exists` function, so tests never depend on what is installed.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::path::Path;

use toolbox_plan::{PackageName, Plan, RemoteAction, Upgrader};

use super::{
    GARUDA_UPDATE_PACKAGE, Operation, Operations, Task, Tool, plan_aur_upgrade, plan_health_check,
    plan_health_fix, plan_now, plan_remote, upgrader_for,
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

const HEALTH_CHECK: &str = "health-check";
const HEALTH_FIX: &str = "health-fix";
const CHROOT: &str = "chroot";

/// The operations in the order the page lists them, the update first.
pub const OPERATION_KEYS: [&str; 5] = [
    UPGRADE_SYSTEM,
    Operation::RemoveOrphans.key(),
    Operation::CleanPackageCache.key(),
    Operation::RefreshKeyring.key(),
    Operation::RemoveDatabaseLock.key(),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationRow {
    pub operation: &'static str,
    pub queued: bool,
}

/// One tool the page opens, or installs first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolRow {
    pub tool: &'static str,
    pub installed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepairRow {
    pub repair: String,
    pub available: bool,
    /// The package that brings its program, named while it is missing.
    pub needs: &'static str,
}

/// Why a program did not start, in the words a user can act on. The
/// system's own text goes beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchCause {
    Missing,
    Denied,
    Other,
}

impl LaunchCause {
    pub const fn of(kind: std::io::ErrorKind) -> Self {
        if matches!(kind, std::io::ErrorKind::NotFound) {
            Self::Missing
        } else if matches!(kind, std::io::ErrorKind::PermissionDenied) {
            Self::Denied
        } else {
            Self::Other
        }
    }
}

/// The upgrader the system has: garuda-update when installed. Without the
/// installed packages, `program_found` says whether its program was there.
pub fn upgrader(snapshot: &Snapshot, program_found: bool) -> Upgrader {
    snapshot.part::<InstalledPackages>().map_or_else(
        || {
            if program_found {
                Upgrader::GarudaUpdate
            } else {
                Upgrader::Pacman
            }
        },
        upgrader_for,
    )
}

pub fn task(id: &str, upgrader: Upgrader) -> Option<Task> {
    [
        Task::RefreshKeyring,
        Task::RemoveDatabaseLock,
        Task::UpgradeSystem { upgrader },
        Task::RemoveOrphans,
        Task::CleanPackageCache,
    ]
    .into_iter()
    .find(|task| task.operation().key() == id)
}

pub fn run_now_plan(id: &str, upgrader: Upgrader) -> Option<Plan> {
    if let Some(task) = task(id, upgrader) {
        return Some(plan_now(task));
    }

    match id {
        HEALTH_CHECK => Some(plan_health_check()),
        HEALTH_FIX => Some(plan_health_fix()),
        AUR_UPGRADE => Some(plan_aur_upgrade()),
        CHROOT => Some(plan_chroot()),
        _ => RemoteAction::ALL
            .into_iter()
            .find(|action| remote_key(*action) == id)
            .map(plan_remote),
    }
}

pub fn operations(changes: &ChangeSet) -> Vec<OperationRow> {
    let queued: BTreeSet<&str> = changes
        .queued::<Operations>()
        .iter()
        .map(|(operation, _)| operation.key())
        .collect();

    OPERATION_KEYS
        .iter()
        .map(|key| OperationRow {
            operation: key,
            queued: queued.contains(key),
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
                tool: tool.id(),
                installed: exists(Path::new(tool.program())),
            };

            let ranking = matches!(tool, Tool::ArchMirrors).then(|| ToolRow {
                tool: CHAOTIC_MIRRORS,
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

    [HEALTH_CHECK, HEALTH_FIX]
        .into_iter()
        .map(|repair| RepairRow {
            repair: repair.to_owned(),
            available: health,
            needs: "garuda-health",
        })
        .chain(RemoteAction::ALL.into_iter().map(|action| RepairRow {
            repair: remote_key(action),
            available: update,
            needs: "garuda-update",
        }))
        .collect()
}

/// Whether the task `id` asks before it runs.
///
/// It removes or reinstalls packages or throws away what the user had
/// (snapshots, `pacman.conf`, the audio setup, old package versions),
/// which no later click brings back.
pub fn asks_first(id: &str) -> bool {
    if [Operation::RemoveOrphans, Operation::CleanPackageCache]
        .iter()
        .any(|operation| operation.key() == id)
    {
        return true;
    }

    RemoteAction::ALL
        .into_iter()
        .filter(|action| remote_key(*action) == id)
        .any(|action| match action {
            RemoteAction::Fix
            | RemoteAction::FullFix
            | RemoteAction::ResetAudio
            | RemoteAction::ResetSnapper
            | RemoteAction::Reinstall => true,
            RemoteAction::Keyring => false,
        })
}

/// The page's id of a remote action (`remote-full-fix`), which differs
/// from the garuda-update argument (`fullfix`).
fn remote_key(action: RemoteAction) -> String {
    let id = match action {
        RemoteAction::Fix => "fix",
        RemoteAction::Keyring => "keyring",
        RemoteAction::FullFix => "full-fix",
        RemoteAction::ResetAudio => "reset-audio",
        RemoteAction::ResetSnapper => "reset-snapper",
        RemoteAction::Reinstall => "reinstall",
    };

    format!("remote-{id}")
}

/// The install of garuda-update, which the Rescue tab's repairs need,
/// while it is missing.
pub fn rescue_install(program: super::Program) -> Option<Plan> {
    let garuda_update = program == super::Program::Found;
    (!garuda_update).then(|| install(&GARUDA_UPDATE_PACKAGE))
}

pub fn install(package: &PackageName) -> Plan {
    Plan::root(crate::domains::install(package))
}

pub fn installed(rows: &[ToolRow], id: &str) -> bool {
    rows.iter().any(|row| row.tool == id && row.installed)
}
