//! Two entries may list the same package or unit (the input methods all
//! bring `fcitx5-configtool`). Turning one of them off must not take it
//! from the other, so an extra stays while another entry that shows on
//! needs it.

use toolbox_catalog::{Check, ToggleCatalog, ToggleEntry};
use toolbox_plan::{PackageName, UnitName};

use super::shown_on;
use crate::domains::{Packages, SystemUnits, UserUnits};
use crate::{ChangeSet, Snapshot, Toggle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extra<'entry> {
    Package(&'entry PackageName),
    Unit(&'entry UnitName),
    UserUnit(&'entry UnitName),
}

impl Extra<'_> {
    fn want(self, wish: Toggle, snapshot: &Snapshot, changes: &mut ChangeSet) {
        match self {
            Self::Package(name) => changes.want::<Packages>(name.clone(), wish, snapshot),
            Self::Unit(unit) => changes.want::<SystemUnits>(unit.clone(), wish, snapshot),
            Self::UserUnit(unit) => changes.want::<UserUnits>(unit.clone(), wish, snapshot),
        }
    }

    fn unqueue(self, changes: &mut ChangeSet) {
        match self {
            Self::Package(name) => changes.remove::<Packages>(name),
            Self::Unit(unit) => changes.remove::<SystemUnits>(unit),
            Self::UserUnit(unit) => changes.remove::<UserUnits>(unit),
        }
    }

    /// Whether `entry` needs this. A package is also needed by the entry
    /// it is the check of: removing it would turn that entry off unasked.
    fn is_listed_by(self, entry: &ToggleEntry) -> bool {
        match self {
            Self::Package(name) => {
                entry.also().contains(name)
                    || matches!(entry.check(), Check::Package(main) if main == name)
            }
            Self::Unit(unit) => entry.also_units().contains(unit),
            Self::UserUnit(unit) => entry.also_user_units().contains(unit),
        }
    }
}

pub fn of(entry: &ToggleEntry) -> impl Iterator<Item = Extra<'_>> {
    let packages = entry.also().iter().map(Extra::Package);
    let units = entry.also_units().iter().map(Extra::Unit);
    let user_units = entry.also_user_units().iter().map(Extra::UserUnit);

    packages.chain(units).chain(user_units)
}

/// Queues the extras of `entry` with the wish of its own package. Call it
/// after that package is queued, so the entry already shows its new state.
pub fn want(
    catalog: &ToggleCatalog,
    entry: &ToggleEntry,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) {
    for extra in of(entry) {
        if wish.is_on() || !is_held(catalog, entry, extra, snapshot, changes) {
            extra.want(wish, snapshot, changes);
        }
    }
}

/// Takes the queued extras of `entry` back, except those another entry
/// that shows on still needs.
pub fn unqueue(
    catalog: &ToggleCatalog,
    entry: &ToggleEntry,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) {
    for extra in of(entry) {
        if !is_held(catalog, entry, extra, snapshot, changes) {
            extra.unqueue(changes);
        }
    }
}

fn is_held(
    catalog: &ToggleCatalog,
    entry: &ToggleEntry,
    extra: Extra<'_>,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> bool {
    catalog
        .entries()
        .filter(|other| other.id() != entry.id())
        .filter(|other| extra.is_listed_by(other))
        .any(|other| shown_on(other, snapshot, changes))
}
