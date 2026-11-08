//! What a page of a toggle catalog shows: the sections with entries this
//! system can switch, each entry with its state and what it needs.

#[cfg(test)]
mod tests;

use toolbox_catalog::{Requires, ToggleCatalog, ToggleEntry, ToggleSection};

use super::{EntryState, is_offered, state};
use crate::{ChangeSet, Snapshot};

/// A section with the entries to show, never empty.
#[derive(Debug)]
pub struct Section<'catalog> {
    pub section: &'catalog ToggleSection,
    pub entries: Vec<Entry<'catalog>>,
}

#[derive(Debug)]
pub struct Entry<'catalog> {
    pub entry: &'catalog ToggleEntry,
    pub state: EntryState,
    /// The entries this one needs, all of them or one of them.
    pub requires: Vec<&'catalog ToggleEntry>,
    pub requires_any: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub on: usize,
    pub total: usize,
}

impl std::iter::Sum for Tally {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::default(), |sum, tally| Self {
            on: sum.on + tally.on,
            total: sum.total + tally.total,
        })
    }
}

impl Section<'_> {
    /// How many entries show as on, the queue counted.
    pub fn on(&self) -> usize {
        self.tally(|_| true).on
    }

    /// The entries `keep` keeps, and how many of them show as on: a
    /// section's count during a search.
    pub fn tally(&self, keep: impl Fn(&Entry<'_>) -> bool) -> Tally {
        self.entries
            .iter()
            .filter(|entry| keep(entry))
            .fold(Tally::default(), |tally, entry| Tally {
                on: tally.on + usize::from(entry.state.shown_on()),
                total: tally.total + 1,
            })
    }
}

impl Entry<'_> {
    /// An entry that is on can always be turned off.
    pub const fn changeable(&self) -> bool {
        self.state.kept.is_none() && (self.state.available || self.state.shown_on())
    }

    pub const fn known(&self) -> bool {
        self.state.system.is_some()
    }
}

/// The sections in catalog order. Entries for other desktops than
/// `desktops` (the session's `XDG_CURRENT_DESKTOP`) and entries the system
/// cannot install are left out.
pub fn sections<'catalog>(
    catalog: &'catalog ToggleCatalog,
    snapshot: &Snapshot,
    changes: &ChangeSet,
    desktops: &[String],
) -> Vec<Section<'catalog>> {
    catalog
        .sections()
        .iter()
        .map(|section| Section {
            section,
            entries: section
                .entries()
                .iter()
                .filter(|entry| entry.is_for_desktop(desktops))
                .filter(|entry| is_offered(entry, snapshot, changes))
                .map(|entry| Entry {
                    entry,
                    state: state(catalog, entry, snapshot, changes),
                    requires: entry
                        .requires()
                        .ids()
                        .iter()
                        .filter_map(|id| catalog.entry(id))
                        .collect(),
                    requires_any: matches!(entry.requires(), Requires::Any(_)),
                })
                .collect(),
        })
        .filter(|section| !section.entries.is_empty())
        .collect()
}

/// How many entries show no state, because what they check was not read.
pub fn unknown(sections: &[Section<'_>]) -> usize {
    sections
        .iter()
        .flat_map(|section| &section.entries)
        .filter(|entry| !entry.known())
        .count()
}
