#[cfg(test)]
mod tests;

pub mod opening;
pub mod page;

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use toolbox_plan::{
    KnownPackage, PackageName, Plan, PlannedAction, RemoteAction, RootAction, Toggle, Upgrader,
    UserAction,
};

use self::page::upgrader;
use crate::domain::{Domain, Stage, Staged, Summary};
use crate::domains::InstalledPackages;
use crate::features::live::plan_chroot;
use crate::{ChangeSet, Snapshot, SummaryCode};

pub static GARUDA_UPDATE_PACKAGE: LazyLock<PackageName> =
    LazyLock::new(|| const { KnownPackage::new("garuda-update") }.name());

pub static RATE_MIRRORS_PACKAGE: LazyLock<PackageName> =
    LazyLock::new(|| const { KnownPackage::new("rate-mirrors") }.name());

/// Cleaning up comes after every root stage that installs or removes
/// packages, and before the user's session so it adds no second helper
/// start.
const CLEANUP: Stage = Stage::new(Stage::USER_SESSION.rank() - 50);

/// The variant order is the run order: keys and the lock first, so the upgrade can work, then cleaning up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    RefreshKeyring,
    RemoveDatabaseLock,
    UpgradeSystem,
    RemoveOrphans,
    CleanPackageCache,
}

/// A second upgrade replaces the queued one, so the Review page lists one row and its key removes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Task {
    RefreshKeyring,
    RemoveDatabaseLock,
    UpgradeSystem { upgrader: Upgrader },
    RemoveOrphans,
    CleanPackageCache,
}

/// Whether garuda-update's program is there, for a snapshot that does not
/// list the installed packages yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Program {
    Found,
    Missing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunNow {
    UpgradeSystem,
    /// paru asks its questions in the Console's terminal.
    AurUpgrade,
    HealthFix,
    /// The findings show in the Console.
    HealthCheck,
    /// A root shell in the installed system, from the live ISO.
    Chroot,
}

impl Operation {
    const fn stage(self) -> Stage {
        match self {
            Self::RemoveOrphans | Self::CleanPackageCache => CLEANUP,
            Self::RefreshKeyring | Self::RemoveDatabaseLock | Self::UpgradeSystem => {
                Stage::REPOSITORIES
            }
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::RefreshKeyring => "refresh-keyring",
            Self::RemoveDatabaseLock => "remove-database-lock",
            Self::UpgradeSystem => "upgrade-system",
            Self::RemoveOrphans => "remove-orphans",
            Self::CleanPackageCache => "clean-package-cache",
        }
    }

    /// The code of its sentence, spelled as its key.
    pub const fn code(self) -> SummaryCode {
        match self {
            Self::RefreshKeyring => SummaryCode::RefreshKeyring,
            Self::RemoveDatabaseLock => SummaryCode::RemoveDatabaseLock,
            Self::UpgradeSystem => SummaryCode::UpgradeSystem,
            Self::RemoveOrphans => SummaryCode::RemoveOrphans,
            Self::CleanPackageCache => SummaryCode::CleanPackageCache,
        }
    }
}

impl Task {
    pub const fn operation(self) -> Operation {
        match self {
            Self::RefreshKeyring => Operation::RefreshKeyring,
            Self::RemoveDatabaseLock => Operation::RemoveDatabaseLock,
            Self::UpgradeSystem { upgrader: _ } => Operation::UpgradeSystem,
            Self::RemoveOrphans => Operation::RemoveOrphans,
            Self::CleanPackageCache => Operation::CleanPackageCache,
        }
    }

    pub const fn action(self) -> RootAction {
        match self {
            Self::RefreshKeyring => RootAction::RefreshKeyring,
            Self::RemoveDatabaseLock => RootAction::RemoveDatabaseLock,
            Self::UpgradeSystem { upgrader } => RootAction::UpgradeSystem { upgrader },
            Self::RemoveOrphans => RootAction::RemoveOrphans,
            Self::CleanPackageCache => RootAction::CleanPackageCache,
        }
    }
}

impl Program {
    pub const fn of_found(found: bool) -> Self {
        if found { Self::Found } else { Self::Missing }
    }
}

impl RunNow {
    #[must_use = "a plan does nothing until it is run"]
    pub fn plan(self, snapshot: &Snapshot, program: Program) -> Plan {
        let garuda_update = program == Program::Found;
        match self {
            Self::UpgradeSystem => plan_now(upgrade(snapshot, garuda_update)),
            Self::AurUpgrade => plan_aur_upgrade(),
            Self::HealthFix => plan_health_fix(),
            Self::HealthCheck => plan_health_check(),
            Self::Chroot => plan_chroot(),
        }
    }
}

/// The queued operations. They are not system state, so nothing a probe
/// sees fulfils them: they stay until removed or until their steps
/// succeeded ([`crate::ChangeSet::drop_done_one_offs`]).
pub struct Operations;

impl Operations {
    pub fn want(changes: &mut ChangeSet, task: Task, snapshot: &Snapshot) {
        changes.want::<Self>(task.operation(), task, snapshot);
    }
}

impl Domain for Operations {
    type Item = Operation;
    type State = ();
    type Wish = Task;

    const ID: &'static str = "operations";
    const ONE_OFF: bool = true;
    const STAGE: Stage = Stage::REPOSITORIES;

    fn state(_snapshot: &Snapshot) -> Option<&()> {
        Some(&())
    }

    fn fits(item: &Operation, wish: &Task) -> bool {
        wish.operation() == *item
    }

    fn is_satisfied(_item: &Operation, _wish: &Task, (): &()) -> bool {
        false
    }

    /// The queue holds them sorted, which is their run order within a
    /// stage.
    fn plan(changes: &[(&Operation, &Task)]) -> Vec<Staged> {
        changes
            .iter()
            .map(|(operation, task)| {
                Staged::at(operation.stage(), PlannedAction::Root(task.action()))
            })
            .collect()
    }

    fn key(item: &Operation) -> String {
        item.key().to_owned()
    }

    fn summary(item: &Operation, _wish: &Task) -> Summary {
        Summary::plain(item.code())
    }
}

/// A graphical program the page starts detached, as the user.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Tool {
    ArchMirrors,
    BtrfsAssistant,
    EditRepositories,
    MergePacdiff,
}

impl Tool {
    pub const ALL: [Self; 4] = [
        Self::ArchMirrors,
        Self::BtrfsAssistant,
        Self::EditRepositories,
        Self::MergePacdiff,
    ];

    pub fn id(self) -> &'static str {
        self.into()
    }

    pub fn from_id(id: &str) -> Option<Self> {
        id.parse().ok()
    }

    pub const fn program(self) -> &'static str {
        match self {
            Self::ArchMirrors => "/usr/bin/reflector-simple",
            Self::BtrfsAssistant => "/usr/bin/btrfs-assistant-launcher",
            Self::EditRepositories => "/usr/bin/pace",
            Self::MergePacdiff => "/usr/lib/garuda/launch-terminal",
        }
    }

    /// pacdiff-merge is a terminal program, so it runs in Garuda's
    /// terminal launcher.
    pub const fn args(self) -> &'static [&'static str] {
        match self {
            Self::ArchMirrors | Self::BtrfsAssistant | Self::EditRepositories => &[],
            Self::MergePacdiff => &["/usr/lib/garuda/pacdiff-merge"],
        }
    }

    /// The package that provides the tool, to offer its installation when
    /// it is missing.
    pub fn package(self) -> PackageName {
        let known = match self {
            Self::ArchMirrors => const { KnownPackage::new("reflector-simple") },
            Self::BtrfsAssistant => const { KnownPackage::new("btrfs-assistant") },
            Self::EditRepositories => const { KnownPackage::new("pace") },
            Self::MergePacdiff => const { KnownPackage::new("meld") },
        };

        known.name()
    }
}

pub fn upgrader_for(installed: &InstalledPackages) -> Upgrader {
    if installed.0.contains(&*GARUDA_UPDATE_PACKAGE) {
        Upgrader::GarudaUpdate
    } else {
        Upgrader::Pacman
    }
}

#[must_use = "a plan does nothing until it is run"]
pub fn plan_now(task: Task) -> Plan {
    Plan::root(task.action())
}

#[must_use = "a plan does nothing until it is run"]
pub fn plan_remote(action: RemoteAction) -> Plan {
    Plan::root(RootAction::GarudaUpdateRemote { action })
}

#[must_use = "a plan does nothing until it is run"]
pub fn plan_health_fix() -> Plan {
    Plan::root(RootAction::GarudaHealthFix)
}

/// The check runs as the user in the Console, so its findings show there.
#[must_use = "a plan does nothing until it is run"]
pub fn plan_health_check() -> Plan {
    Plan::user(UserAction::GarudaHealth)
}

/// paru runs in the Console's terminal without `--noconfirm`, so the user
/// reviews the PKGBUILD diffs and answers its questions there.
#[must_use = "a plan does nothing until it is run"]
pub fn plan_aur_upgrade() -> Plan {
    Plan::user(UserAction::UpgradeAur)
}

pub fn upgrade_queued(changes: &ChangeSet) -> bool {
    changes
        .queued::<Operations>()
        .iter()
        .any(|(operation, _)| *operation == Operation::UpgradeSystem)
}

pub fn queue_upgrade(wish: Toggle, changes: &mut ChangeSet, snapshot: &Snapshot, program: Program) {
    let garuda_update = program == Program::Found;
    let on = wish.is_on();
    if on {
        Operations::want(changes, upgrade(snapshot, garuda_update), snapshot);
    } else {
        changes.remove::<Operations>(&Operation::UpgradeSystem);
    }
}

fn upgrade(snapshot: &Snapshot, garuda_update: bool) -> Task {
    Task::UpgradeSystem {
        upgrader: upgrader(snapshot, garuda_update),
    }
}
