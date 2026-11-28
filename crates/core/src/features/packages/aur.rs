//! The Packages page's AUR list and how its rows stand against the queue.
//!
//! The search looks in names, descriptions and keywords and orders the
//! way people look for a package. Installs are paru builds, removals are
//! the packages domain's.

pub mod build;
#[cfg(test)]
mod tests;
pub mod upgrade;

use std::collections::{BTreeMap, BTreeSet};

use toolbox_plan::PackageName;
pub use toolbox_state::aur::{AurIndex, AurPackage};
use toolbox_state::aur::{AurRoute, RouteKind};
use toolbox_text::OutsideText;

use super::list::RowState;
use super::queue::Refusal;
use crate::domains::{AurPackages, AurWish, BuildOrder, InstalledPackages, Packages};
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
/// once it read the copy, for [`recheck`] and the conflicts of [`toggle`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AurNames {
    pub names: BTreeSet<PackageName>,
    /// The names each package's `conflicts` list names, without versions.
    /// Only packages that name one are listed.
    pub conflicts: BTreeMap<PackageName, Vec<PackageName>>,
}

impl toolbox_state::Part for AurNames {}

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

    /// Every package's name and conflicts, for the snapshot's [`AurNames`].
    pub fn names(&self) -> AurNames {
        let conflicts = self
            .index
            .packages
            .iter()
            .filter_map(|package| {
                let names: Vec<PackageName> = package
                    .conflicts
                    .iter()
                    .filter_map(|conflict| PackageName::new(conflict_name(conflict.as_str())).ok())
                    .collect();

                (!names.is_empty()).then(|| (package.name.clone(), names))
            })
            .collect();

        AurNames {
            names: self
                .index
                .packages
                .iter()
                .map(|package| package.name.clone())
                .collect(),
            conflicts,
        }
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

/// The name a `conflicts` entry names, before any version constraint.
fn conflict_name(text: &str) -> &str {
    text.split(['<', '>', '='])
        .next()
        .unwrap_or_default()
        .trim()
}

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
            let queued = build_wish(package, snapshot)?;
            changes.want::<AurPackages>(package.clone(), queued, snapshot);
        }
        (Toggle::Off, true) => {
            super::queue::may_remove(package, changes, snapshot)?;
            changes.want::<Packages>(package.clone(), Toggle::Off, snapshot);
        }
        (Toggle::Off, false) => changes.remove::<AurPackages>(package),
    }

    Ok(())
}

/// Whether a new AUR build takes the split path: the helper accepts the
/// calling user's build folder. A build that waits for its order is then
/// built by the app, else by paru.
pub fn takes_split_path(snapshot: &Snapshot) -> bool {
    matches!(snapshot.part::<AurRoute>(), Some(AurRoute::Accepted))
}

/// The reason the helper refuses the calling user's build folder, which is
/// why a setup takes paru's own path. `None` while the route is accepted or
/// not read.
pub fn route_refusal(snapshot: &Snapshot) -> Option<RouteKind> {
    match snapshot.part::<AurRoute>() {
        Some(AurRoute::Refused { kind, detail: _ }) => Some(*kind),
        Some(AurRoute::Accepted) | None => None,
    }
}

/// The wish a new build of `package` queues: the split path when the helper
/// accepts the route, else paru's own build. Only the split path checks
/// conflicts, as paru asks the user itself when a conflict comes up.
///
/// # Errors
///
/// [`Refusal::Unread`] without the route answer, or without the AUR's
/// names on the split path, [`Refusal::Conflicts`] for an installed package
/// the split path would conflict with.
fn build_wish(
    package: &PackageName,
    snapshot: &Snapshot,
) -> Result<AurWish, Refused<Refusal, PackageName>> {
    match snapshot.part::<AurRoute>() {
        None => Err(Refusal::Unread.of(package)),
        Some(AurRoute::Refused { kind: _, detail: _ }) => Ok(AurWish::Paru),
        Some(AurRoute::Accepted) => {
            let names = snapshot
                .part::<AurNames>()
                .ok_or_else(|| Refusal::Unread.of(package))?;

            let installed = snapshot
                .part::<InstalledPackages>()
                .ok_or_else(|| Refusal::Unread.of(package))?;

            let conflict = names
                .conflicts
                .get(package)
                .and_then(|conflicts| conflicts.iter().find(|name| installed.0.contains(*name)));

            conflict.map_or(
                Ok(AurWish::Waiting),
                |with| Err(Refusal::Conflicts.of(with)),
            )
        }
    }
}

/// Gives the queued build of `package` its `order`, or waits for it again
/// without one. A paru build and a package that is not queued keep their
/// wish.
pub fn give_order(
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
    package: &PackageName,
    order: Option<BuildOrder>,
) {
    let wish = match (changes.wish::<AurPackages>(package), order) {
        (None | Some(AurWish::Paru), _) => return,
        (Some(_), Some(order)) => AurWish::Built(order),
        (Some(_), None) => AurWish::Waiting,
    };

    changes.want::<AurPackages>(package.clone(), wish, snapshot);
}

/// The queued AUR builds that need a build order read before a plan: the
/// waiting ones and the built ones, as an apply reads every order again.
pub fn needing_orders(changes: &ChangeSet) -> Vec<PackageName> {
    changes
        .queued::<AurPackages>()
        .into_iter()
        .filter(|(_, wish)| !matches!(wish, AurWish::Paru))
        .map(|(package, _)| package)
        .collect()
}

/// Drops a queued AUR build of a name the AUR's copy lacks, as one from
/// the session bus or a saved file may hold. `None` until the snapshot
/// holds the copy's names.
pub fn recheck(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    let names = snapshot.part::<AurNames>()?;

    Some(drop_refused::<AurPackages>(
        changes,
        "AUR build",
        |item, _, _| (!names.names.contains(item)).then_some("the AUR has no such package"),
    ))
}
