#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use toolbox_plan::PackageName;
use toolbox_state::packages::{IndexEntry, LocalNode, PackageIndex};

use super::graph::orphans;
use crate::domains::{InstalledPackages, Packages};
use crate::features::search::{Needle, folded};
use crate::{ChangeSet, Snapshot, Toggle};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, strum::IntoStaticStr, strum::EnumIter,
)]
#[strum(serialize_all = "kebab-case")]
pub enum Filter {
    #[default]
    Installed,
    Updates,
    Orphans,
    All,
    /// The AUR's own list, which never mixes with the repositories'.
    Aur,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub installed: usize,
    pub updates: usize,
    pub orphans: usize,
    pub all: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct View {
    pub rows: Vec<usize>,
    pub counts: Counts,
}

/// The index with its texts folded once, so a keystroke compares without
/// a new string per package.
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    index: PackageIndex,
    folded: Vec<String>,
    orphans: BTreeSet<PackageName>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowState {
    pub installed: bool,
    pub queued: Option<Toggle>,
}

impl Catalog {
    /// Folding every text takes a while for the whole index, so this runs
    /// where the index is read, off the frame.
    pub fn new(mut index: PackageIndex) -> Self {
        index
            .entries
            .sort_by(|left, right| left.name.cmp(&right.name));
        let folded = index
            .entries
            .iter()
            .map(|entry| {
                format!(
                    "{}\n{}",
                    folded(entry.name.as_str()),
                    folded(&entry.description)
                )
            })
            .collect();

        let orphans = orphans(&index.local);

        Self {
            index,
            folded,
            orphans,
        }
    }

    pub fn entries(&self) -> &[IndexEntry] {
        &self.index.entries
    }

    pub fn local(&self) -> &[LocalNode] {
        &self.index.local
    }

    pub fn entry(&self, name: &PackageName) -> Option<&IndexEntry> {
        let entries = &self.index.entries;

        let place = entries
            .binary_search_by(|entry| entry.name.cmp(name))
            .ok()?;

        entries.get(place)
    }

    pub fn is_orphan(&self, name: &PackageName) -> bool {
        self.orphans.contains(name)
    }

    pub fn view(&self, filter: Filter, search: &str, updates: &BTreeSet<PackageName>) -> View {
        let needle = Needle::new(search);

        let mut view = View::default();

        for (place, (entry, text)) in self.index.entries.iter().zip(&self.folded).enumerate() {
            if !text.contains(needle.as_str()) {
                continue;
            }

            let installed = entry.installed.is_some();
            let update = installed && updates.contains(&entry.name);
            let orphan = self.orphans.contains(&entry.name);
            let counts = &mut view.counts;
            counts.all += 1;
            counts.installed += usize::from(installed);
            counts.updates += usize::from(update);
            counts.orphans += usize::from(orphan);

            let shown = match filter {
                Filter::Installed => installed,
                Filter::Updates => update,
                Filter::Orphans => orphan,
                Filter::All => true,
                Filter::Aur => false,
            };

            if shown {
                view.rows.push(place);
            }
        }

        view
    }
}

impl Filter {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl RowState {
    pub fn shown_installed(self) -> bool {
        self.queued.map_or(self.installed, Toggle::is_on)
    }
}

pub fn row_state(package: &PackageName, snapshot: &Snapshot, changes: &ChangeSet) -> RowState {
    RowState {
        installed: snapshot
            .part::<InstalledPackages>()
            .is_some_and(|installed| installed.0.contains(package)),
        queued: changes.wish::<Packages>(package).copied(),
    }
}
