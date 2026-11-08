//! The logic behind the Discover page.
//!
//! The apps of the category catalogs (`apps/<category>.toml`) show as one
//! list per category, with a search over all of them. A click queues the
//! app's packages like any toggle catalog entry, so the rules of
//! [`super::toggles`] apply unchanged.

#[cfg(test)]
mod tests;

use toolbox_catalog::{Check, ToggleCatalog, ToggleEntry, ToggleSection};
use toolbox_plan::PackageName;

use super::search::{Needle, folded};
use super::toggles::{self, is_offered};
use crate::{ChangeSet, Snapshot, Toggle};

/// Endings of package builds and of Garuda's meta packages, whose icon
/// carries the plain name.
const BUILD_SUFFIXES: [&str; 4] = ["-bin", "-git", "-appimage", "-meta"];

/// The category catalogs, in page order. Entry ids are unique across them
/// (`cargo xtask catalog-validate` checks it).
#[derive(Clone, Debug, Default)]
pub struct Apps {
    catalogs: Vec<ToggleCatalog>,
    /// One per app, in the order of [`Apps::all`].
    folded: Vec<Folded>,
}

/// An app's texts folded the way [`Needle`] compares, made once: a
/// search and a sort then need no new string per app and keystroke.
#[derive(Clone, Debug, Default)]
pub struct Folded {
    title: String,
    /// Title, description and packages, a line each.
    text: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Hit<'apps> {
    pub catalog: &'apps ToggleCatalog,
    pub category: &'apps ToggleSection,
    pub entry: &'apps ToggleEntry,
    pub folded: &'apps Folded,
}

#[derive(Clone, Copy, Debug)]
pub struct Category<'apps> {
    pub section: &'apps ToggleSection,
    pub count: usize,
}

impl Apps {
    pub fn new(catalogs: Vec<ToggleCatalog>) -> Self {
        let folded = entries(&catalogs)
            .map(|(_, _, entry)| Folded::of(entry))
            .collect();

        Self { catalogs, folded }
    }

    pub fn all(&self) -> impl Iterator<Item = Hit<'_>> {
        entries(&self.catalogs)
            .zip(&self.folded)
            .map(|((catalog, category, entry), folded)| Hit {
                catalog,
                category,
                entry,
                folded,
            })
    }

    /// The categories with the number of offered apps `search` matches,
    /// all of them for an empty search. A category stays with none while
    /// it offers any app, so its chip does not vanish during a search.
    pub fn categories_matching(
        &self,
        search: &str,
        snapshot: &Snapshot,
        changes: &ChangeSet,
    ) -> Vec<Category<'_>> {
        let needle = Needle::new(search);
        let mut tallies: Vec<(&ToggleSection, usize)> = Vec::new();
        for hit in self
            .all()
            .filter(|hit| is_offered(hit.entry, snapshot, changes))
        {
            let matched = usize::from(hit.matches(&needle));
            match tallies.last_mut() {
                Some((section, count)) if std::ptr::eq(*section, hit.category) => *count += matched,
                _ => tallies.push((hit.category, matched)),
            }
        }

        tallies
            .into_iter()
            .map(|(section, count)| Category { section, count })
            .collect()
    }

    /// The offered apps of `category` (`None` for every category) that
    /// `search` matches, all of them for an empty search.
    pub fn filtered(
        &self,
        category: Option<&str>,
        search: &str,
        snapshot: &Snapshot,
        changes: &ChangeSet,
    ) -> Vec<Hit<'_>> {
        let needle = Needle::new(search);

        self.all()
            .filter(|hit| category.is_none_or(|id| hit.category.id() == id))
            .filter(|hit| hit.matches(&needle))
            .filter(|hit| is_offered(hit.entry, snapshot, changes))
            .collect()
    }

    /// Every app this system can install or remove, in page order, with
    /// its place in [`Apps::all`].
    pub fn offered<'apps>(
        &'apps self,
        snapshot: &'apps Snapshot,
        changes: &'apps ChangeSet,
    ) -> impl Iterator<Item = (usize, Hit<'apps>)> {
        self.all()
            .enumerate()
            .filter(|(_, hit)| is_offered(hit.entry, snapshot, changes))
    }

    /// Queues app `id` on or off. Returns `false` for an unknown id or a
    /// refused click.
    pub fn toggle(
        &self,
        id: &str,
        wish: Toggle,
        snapshot: &Snapshot,
        changes: &mut ChangeSet,
    ) -> bool {
        self.catalogs
            .iter()
            .find(|catalog| catalog.entry(id).is_some())
            .is_some_and(|catalog| toggles::toggle(catalog, id, wish, snapshot, changes).is_ok())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum AppOrder {
    /// By name without regard to case, which is also how a search over
    /// several categories mixes them.
    #[default]
    Name,
    /// Installed or queued apps first, each group by name.
    Selected,
}

impl AppOrder {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl Folded {
    fn of(entry: &ToggleEntry) -> Self {
        let title = folded(entry.title());
        let mut text = format!("{title}\n{}", folded(entry.description()));

        for package in packages(entry) {
            text.push('\n');
            text.push_str(&folded(package.as_str()));
        }

        Self { title, text }
    }
}

impl Hit<'_> {
    /// Whether the app's title, description or one of its packages holds
    /// `needle`.
    pub fn matches(&self, needle: &Needle) -> bool {
        self.folded.text.contains(needle.as_str())
    }

    /// Whether the app's title holds `needle`, for the command palette,
    /// which names apps by their title alone.
    pub fn title_matches(&self, needle: &Needle) -> bool {
        self.folded.title.contains(needle.as_str())
    }
}

fn entries(
    catalogs: &[ToggleCatalog],
) -> impl Iterator<Item = (&ToggleCatalog, &ToggleSection, &ToggleEntry)> {
    catalogs.iter().flat_map(|catalog| {
        catalog.sections().iter().flat_map(move |category| {
            category
                .entries()
                .iter()
                .map(move |entry| (catalog, category, entry))
        })
    })
}

/// The package an app installs, `None` for an entry of another kind.
pub const fn package(entry: &ToggleEntry) -> Option<&PackageName> {
    match entry.check() {
        Check::Package(name) => Some(name),
        Check::Service(_) | Check::UserService(_) | Check::Group(_) | Check::UserConfig(_) => None,
    }
}

/// The icon theme name of a package: its name without the ending of a
/// binary, development, `AppImage` or meta build (`firefox-bin` shows
/// `firefox`, `virtualbox-meta` shows `virtualbox`).
pub fn icon_name(package: &PackageName) -> &str {
    let name = package.as_str();

    BUILD_SUFFIXES
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .filter(|base| !base.is_empty())
        .unwrap_or(name)
}

fn packages(entry: &ToggleEntry) -> impl Iterator<Item = &PackageName> {
    package(entry).into_iter().chain(entry.also())
}

pub fn arrange(hits: &mut [Hit<'_>], order: AppOrder, snapshot: &Snapshot, changes: &ChangeSet) {
    let selected = |hit: &Hit<'_>| {
        order == AppOrder::Selected
            && toggles::state(hit.catalog, hit.entry, snapshot, changes).shown_on()
    };

    hits.sort_by_cached_key(|hit| (!selected(hit), hit.folded.title.as_str()));
}
