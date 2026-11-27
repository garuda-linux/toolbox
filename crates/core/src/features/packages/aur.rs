//! The Packages page's AUR list and how its rows stand against the queue.
//!
//! The search looks in names, descriptions and keywords and orders the
//! way people look for a package. Installs are paru builds, removals are
//! the packages domain's.

pub mod build;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use toolbox_plan::PackageName;
pub use toolbox_state::aur::{AurIndex, AurPackage};
use toolbox_text::OutsideText;

use super::list::RowState;
use super::queue::Refusal;
use crate::domains::{AurPackages, AurWish, InstalledPackages, Packages};
use crate::features::search::ranked::Rank as SearchRank;
use crate::features::search::{Needle, folded};
use crate::features::{Refused, drop_refused};
use crate::{ChangeSet, Snapshot, Toggle};

/// The dump with its texts folded once, so a keystroke compares without
/// a new string per package.
#[derive(Clone, Debug, Default)]
pub struct AurCatalog {
    index: AurIndex,
    names: Vec<String>,
    texts: Vec<String>,
    /// The places in the most popular first, so a search needs no sort.
    popular: Vec<usize>,
}

/// The names of the AUR's copy, a part the window adds to its snapshot
/// once it read the copy, for [`recheck`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AurNames(pub BTreeSet<PackageName>);

/// How a name stands against the search, the best first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    Exact,
    Starts,
    Elsewhere,
}

impl AurCatalog {
    /// Folding a hundred thousand texts takes a while, so this runs where
    /// the dump is read, off the frame.
    pub fn new(mut index: AurIndex) -> Self {
        index
            .packages
            .sort_by(|left, right| left.name.cmp(&right.name));

        let names = index
            .packages
            .iter()
            .map(|package| folded(package.name.as_str()))
            .collect();

        let texts = index
            .packages
            .iter()
            .map(|package| {
                format!(
                    "{}\n{}\n{}",
                    folded(package.name.as_str()),
                    folded(&package.description),
                    folded(&keywords(&package.keywords))
                )
            })
            .collect();

        let mut popular: Vec<usize> = (0..index.packages.len()).collect();
        let popularity = |place: &usize| {
            index
                .packages
                .get(*place)
                .map_or(0.0_f64, |package| package.popularity)
        };

        popular.sort_by(|left, right| popularity(right).total_cmp(&popularity(left)));

        Self {
            index,
            names,
            texts,
            popular,
        }
    }

    /// Every package, sorted by name.
    pub fn entries(&self) -> &[AurPackage] {
        &self.index.packages
    }

    pub fn entry(&self, name: &PackageName) -> Option<&AurPackage> {
        let packages = &self.index.packages;
        let place = packages
            .binary_search_by(|package| package.name.cmp(name))
            .ok()?;

        packages.get(place)
    }

    /// Every package's name, for the snapshot's [`AurNames`].
    pub fn names(&self) -> AurNames {
        AurNames(
            self.index
                .packages
                .iter()
                .map(|package| package.name.clone())
                .collect(),
        )
    }

    /// Seconds since the epoch when the copy was downloaded.
    pub const fn downloaded(&self) -> i64 {
        self.index.downloaded
    }

    /// The last download of a newer copy failed.
    pub const fn stale(&self) -> bool {
        self.index.stale
    }

    /// Places in [`AurCatalog::entries`], the most popular first.
    pub fn ranked<'a>(
        &'a self,
        needle: &'a Needle,
    ) -> impl Iterator<Item = (SearchRank, usize)> + 'a {
        self.popular.iter().filter_map(|&place| {
            let rank = needle.rank_folded(self.names.get(place)?, self.texts.get(place)?)?;
            Some((rank, place))
        })
    }

    /// The places in [`AurCatalog::entries`] that `search` matches: the
    /// exact name first, then names that start with it, then the rest,
    /// each part the most popular first.
    pub fn view(&self, search: &str) -> Vec<usize> {
        let needle = Needle::new(search);
        let wanted = needle.as_str();
        let (mut exact, mut starts, mut elsewhere) = (Vec::new(), Vec::new(), Vec::new());

        for &place in &self.popular {
            let (Some(name), Some(text)) = (self.names.get(place), self.texts.get(place)) else {
                continue;
            };

            if text.contains(wanted) {
                let part = match rank(name, wanted) {
                    Rank::Exact => &mut exact,
                    Rank::Starts => &mut starts,
                    Rank::Elsewhere => &mut elsewhere,
                };

                part.push(place);
            }
        }

        exact.extend(starts);
        exact.extend(elsewhere);
        exact
    }
}

impl toolbox_state::Part for AurNames {}

fn keywords(keywords: &[OutsideText]) -> String {
    keywords
        .iter()
        .map(OutsideText::as_str)
        .collect::<Vec<_>>()
        .join("\n")
}

fn rank(name: &str, wanted: &str) -> Rank {
    if wanted.is_empty() {
        Rank::Elsewhere
    } else if name == wanted {
        Rank::Exact
    } else if name.starts_with(wanted) {
        Rank::Starts
    } else {
        Rank::Elsewhere
    }
}

/// How `package` of the AUR list stands: installed as the snapshot says,
/// a queued build as `On`, a queued removal as `Off`.
pub fn row_state(package: &PackageName, snapshot: &Snapshot, changes: &ChangeSet) -> RowState {
    let build = changes
        .wish::<AurPackages>(package)
        .map(|_queued| Toggle::On);

    let removal = changes
        .wish::<Packages>(package)
        .copied()
        .filter(|wish| !wish.is_on());

    RowState {
        installed: snapshot
            .part::<InstalledPackages>()
            .is_some_and(|installed| installed.0.contains(package)),
        queued: build.or(removal),
    }
}

/// A click on an AUR row: `On` queues a build or takes a removal back,
/// `Off` queues a removal or takes a build back.
///
/// # Errors
///
/// [`Refusal::Unread`] while the installed packages are not known.
pub fn toggle(
    package: &PackageName,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refused<Refusal, PackageName>> {
    let installed = snapshot
        .part::<InstalledPackages>()
        .ok_or_else(|| Refusal::Unread.of(package))?
        .0
        .contains(package);

    match (wish, installed) {
        (Toggle::On, true) => changes.remove::<Packages>(package),
        (Toggle::On, false) => {
            changes.want::<AurPackages>(package.clone(), AurWish::Paru, snapshot);
        }
        (Toggle::Off, true) => {
            super::queue::may_remove(package, changes, snapshot)?;
            changes.want::<Packages>(package.clone(), Toggle::Off, snapshot);
        }
        (Toggle::Off, false) => changes.remove::<AurPackages>(package),
    }

    Ok(())
}

/// Drops a queued AUR build of a name the AUR's copy lacks, as one from
/// the session bus or a saved file may hold. `None` until the snapshot
/// holds the copy's names.
pub fn recheck(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    let names = snapshot.part::<AurNames>()?;

    Some(drop_refused::<AurPackages>(
        changes,
        "AUR build",
        |item, _, _| (!names.0.contains(item)).then_some("the AUR has no such package"),
    ))
}
