//! The Services page: system and user units and the one-off operations on
//! them.
//!
//! Enabling and masking are states and queue through
//! [`crate::domains::SystemUnits`] and [`crate::domains::SystemUnitMasks`] (and
//! their user twins).

mod states;
#[cfg(test)]
mod tests;
pub mod view;

use std::cmp::Ordering;

pub use states::sub_code;
use toolbox_plan::{
    Plan, PlannedAction, RootAction, TargetList, UnitName, UnitOperation, UnitScope, UserAction,
};
pub use toolbox_state::services::{ActiveState, FileState, UnitInfo};

use super::search::{Needle, Searchable};
use crate::domains::{SystemUnitMasks, SystemUnits, UserUnitMasks, UserUnits};
use crate::{ChangeSet, Snapshot, Toggle};

/// Units the session lives on: stopping one ends the graphical session,
/// the network or the password dialog.
const CRITICAL: [&str; 11] = [
    "display-manager.service",
    "graphical.target",
    "sddm.service",
    "gdm.service",
    "lightdm.service",
    "greetd.service",
    "NetworkManager.service",
    "polkit.service",
    "dbus.service",
    "dbus-broker.service",
    "systemd-logind.service",
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    pub search: String,
    pub shown: Shown,
}

/// Which units the list shows. One is picked at a time, so every choice
/// says what it lets through.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Shown {
    /// Neither stopped nor failed. A unit changing state stays, it is
    /// worth a look.
    #[default]
    Running,
    /// Starts at boot.
    Enabled,
    /// Could start at boot, but does not. A static unit has no such switch.
    Disabled,
    Masked,
    /// Stopped and not failed: what systemd calls `inactive (dead)`.
    Dead,
    Failed,
    All,
}

/// How many units each choice of [`Shown`] lets through.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub running: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub masked: usize,
    pub dead: usize,
    pub failed: usize,
    pub all: usize,
}

/// What the Services page queues for a unit: whether it starts at boot,
/// or whether it is masked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitWish {
    AtBoot(Toggle),
    Masked(Toggle),
}

impl Shown {
    pub const EVERY: [Self; 7] = [
        Self::Running,
        Self::Enabled,
        Self::Disabled,
        Self::Masked,
        Self::Dead,
        Self::Failed,
        Self::All,
    ];

    /// Whether a unit in `active` whose unit file is in `file` is listed.
    /// The command line has only these two facts of a unit.
    pub const fn keeps(self, active: ActiveState, file: FileState) -> bool {
        match self {
            Self::Running => !matches!(active, ActiveState::Inactive | ActiveState::Failed),
            Self::Enabled => matches!(file, FileState::Enabled),
            Self::Disabled => matches!(file, FileState::Disabled),
            Self::Masked => matches!(file, FileState::Masked),
            Self::Dead => matches!(active, ActiveState::Inactive),
            Self::Failed => matches!(active, ActiveState::Failed),
            Self::All => true,
        }
    }

    pub const fn lets_through(self, unit: &UnitInfo) -> bool {
        self.keeps(unit.active, unit.file_state)
    }

    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl Counts {
    pub const fn of(self, shown: Shown) -> usize {
        match shown {
            Shown::Running => self.running,
            Shown::Enabled => self.enabled,
            Shown::Disabled => self.disabled,
            Shown::Masked => self.masked,
            Shown::Dead => self.dead,
            Shown::Failed => self.failed,
            Shown::All => self.all,
        }
    }
}

/// What the list is ordered by. Ties fall back to the name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum UnitOrder {
    #[default]
    Name,
    /// What needs a look first: failed units, then those changing state,
    /// then running ones by their sub state, then the stopped.
    State,
}

impl UnitOrder {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl Searchable for UnitInfo {
    fn texts(&self) -> impl Iterator<Item = &str> {
        [self.name.as_str(), self.description.as_str()].into_iter()
    }
}

/// What each choice of [`Shown`] would list with `search`, so the chips
/// count what a click on them shows.
pub fn counts(units: &[UnitInfo], search: &str) -> Counts {
    let needle = Needle::new(search);

    units
        .iter()
        .filter(|unit| needle.finds(*unit))
        .fold(Counts::default(), |counts, unit| Counts {
            running: counts.running + usize::from(Shown::Running.lets_through(unit)),
            enabled: counts.enabled + usize::from(Shown::Enabled.lets_through(unit)),
            disabled: counts.disabled + usize::from(Shown::Disabled.lets_through(unit)),
            masked: counts.masked + usize::from(Shown::Masked.lets_through(unit)),
            dead: counts.dead + usize::from(Shown::Dead.lets_through(unit)),
            failed: counts.failed + usize::from(Shown::Failed.lets_through(unit)),
            all: counts.all + 1,
        })
}

/// The places in `units` of the ones `filter` lets through, in `order`.
pub fn pick(units: &[UnitInfo], filter: &Filter, order: UnitOrder) -> Vec<usize> {
    let mut listed = shown(units, filter);

    if order == UnitOrder::State {
        listed.sort_by(|(_, left), (_, right)| by_state(left, right));
    }

    listed.into_iter().map(|(at, _)| at).collect()
}

fn shown<'units>(units: &'units [UnitInfo], filter: &Filter) -> Vec<(usize, &'units UnitInfo)> {
    let search = Needle::new(&filter.search);

    let mut shown: Vec<(usize, &UnitInfo)> = units
        .iter()
        .enumerate()
        .filter(|(_, unit)| search.finds(*unit))
        .filter(|(_, unit)| filter.shown.lets_through(unit))
        .collect();

    // People scan the list alphabetically, so case must not split it in two.
    shown.sort_by_cached_key(|(_, unit)| (unit.name.as_str().to_lowercase(), unit.name.clone()));
    shown
}

fn by_state(left: &UnitInfo, right: &UnitInfo) -> Ordering {
    rank(left.active)
        .cmp(&rank(right.active))
        .then_with(|| left.sub_state.cmp(&right.sub_state))
}

const fn rank(active: ActiveState) -> u8 {
    match active {
        ActiveState::Failed => 0,
        ActiveState::Activating
        | ActiveState::Deactivating
        | ActiveState::Reloading
        | ActiveState::Maintenance
        | ActiveState::Refreshing => 1,
        ActiveState::Other => 2,
        ActiveState::Active => 3,
        ActiveState::Inactive => 4,
    }
}

/// Whether `operation` would leave a unit in `active` as it is: starting a
/// running unit or stopping a stopped one. A failed unit counts as
/// stopped.
pub const fn does_nothing(operation: UnitOperation, active: ActiveState) -> bool {
    match operation {
        UnitOperation::Start => matches!(
            active,
            ActiveState::Active | ActiveState::Reloading | ActiveState::Activating
        ),
        UnitOperation::Stop => matches!(active, ActiveState::Inactive | ActiveState::Failed),
        UnitOperation::Restart | UnitOperation::Reload => false,
    }
}

/// Whether the session lives on `unit`, so masking it asks first.
pub fn critical(unit: &UnitName) -> bool {
    CRITICAL.contains(&unit.as_str())
}

/// Whether `operation` on `unit` asks first: stopping or restarting what
/// the session lives on.
pub fn asks_first(operation: UnitOperation, unit: &UnitName) -> bool {
    matches!(operation, UnitOperation::Stop | UnitOperation::Restart) && critical(unit)
}

/// One operation run at once in the Console, past the queue.
#[must_use = "a plan does nothing until it is run"]
pub fn plan_control(operation: UnitOperation, scope: UnitScope, unit: UnitName) -> Plan {
    let units = TargetList::one(unit);

    Plan::new([match scope {
        UnitScope::System => PlannedAction::Root(RootAction::ControlUnits { operation, units }),
        UnitScope::User => PlannedAction::User(UserAction::ControlUserUnits { operation, units }),
    }])
}

/// The unit's journal in the Console. Reading it needs no root.
#[must_use = "a plan does nothing until it is run"]
pub fn plan_logs(scope: UnitScope, unit: UnitName) -> Plan {
    Plan::user(UserAction::UnitLogs { unit, scope })
}

/// Queues `wish` for `unit` of `scope`. One the system already has is not
/// queued.
pub fn queue(
    changes: &mut ChangeSet,
    scope: UnitScope,
    unit: UnitName,
    wish: UnitWish,
    snapshot: &Snapshot,
) {
    match (wish, scope) {
        (UnitWish::AtBoot(on), UnitScope::System) => {
            changes.want::<SystemUnits>(unit, on, snapshot);
        }
        (UnitWish::AtBoot(on), UnitScope::User) => changes.want::<UserUnits>(unit, on, snapshot),
        (UnitWish::Masked(on), UnitScope::System) => {
            changes.want::<SystemUnitMasks>(unit, on, snapshot);
        }
        (UnitWish::Masked(on), UnitScope::User) => {
            changes.want::<UserUnitMasks>(unit, on, snapshot);
        }
    }
}
