//! The Services page: system and user units and the one-off operations on
//! them.
//!
//! Enabling and masking are states and queue through
//! [`crate::domains::SystemUnits`] and [`crate::domains::SystemUnitMasks`] (and
//! their user twins).

mod states;
#[cfg(test)]
mod tests;

use std::cmp::Ordering;

pub use states::sub_code;
use toolbox_plan::{
    Plan, PlannedAction, RootAction, TargetList, UnitName, UnitOperation, UnitScope, UserAction,
};
pub use toolbox_state::services::{ActiveState, FileState, UnitInfo};

use super::search::{Needle, Searchable};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub search: String,
    pub include_disabled: bool,
    pub hide_inactive: bool,
}

/// The page starts with what runs: disabled units left out and inactive
/// ones hidden.
impl Default for Filter {
    fn default() -> Self {
        Self {
            search: String::new(),
            include_disabled: false,
            hide_inactive: true,
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

/// The places in `units` of the ones `filter` lets through, in `order`.
/// Failed units stay when inactive ones are hidden, they are what the
/// user most likely looks for.
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
        .filter(|(_, unit)| filter.include_disabled || unit.file_state != FileState::Disabled)
        .filter(|(_, unit)| !filter.hide_inactive || unit.active != ActiveState::Inactive)
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
