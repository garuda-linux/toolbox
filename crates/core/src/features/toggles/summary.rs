//! What the entries of one toggle catalog have waiting in the queue, for
//! the last step of the Setup Assistant. Changes queued on other pages
//! are left out, the Review page lists those.

use std::collections::BTreeSet;

use toolbox_catalog::{ToggleCatalog, ToggleEntry};
use toolbox_plan::UnitName;

use super::queued;
use crate::domains::{SystemUnits, UserUnits};
use crate::{ChangeSet, Snapshot, Toggle};

/// The queued part of a catalog, entries in file order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary<'catalog> {
    /// Entries queued to be turned on.
    pub install: Vec<&'catalog ToggleEntry>,
    /// Entries queued to be turned off.
    pub remove: Vec<&'catalog ToggleEntry>,
    /// System units the entries enable.
    pub units: BTreeSet<&'catalog UnitName>,
    /// Units of the user's service manager the entries enable.
    pub user_units: BTreeSet<&'catalog UnitName>,
}

impl Summary<'_> {
    /// Whether nothing of the catalog waits in the queue.
    pub fn is_empty(&self) -> bool {
        self.install.is_empty()
            && self.remove.is_empty()
            && self.units.is_empty()
            && self.user_units.is_empty()
    }
}

pub fn summary<'catalog>(
    catalog: &'catalog ToggleCatalog,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> Summary<'catalog> {
    let mut summary = Summary::default();

    for entry in catalog.entries() {
        match queued(entry, snapshot, changes) {
            Some(Toggle::On) => summary.install.push(entry),
            Some(Toggle::Off) => summary.remove.push(entry),
            None => {}
        }

        summary.units.extend(
            entry
                .also_units()
                .iter()
                .filter(|unit| changes.wish::<SystemUnits>(unit) == Some(&Toggle::On)),
        );
        summary.user_units.extend(
            entry
                .also_user_units()
                .iter()
                .filter(|unit| changes.wish::<UserUnits>(unit) == Some(&Toggle::On)),
        );
    }

    summary
}
