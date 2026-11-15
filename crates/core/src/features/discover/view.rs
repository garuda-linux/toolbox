#[cfg(test)]
mod tests;

use toolbox_catalog::{ToggleCatalog, ToggleSection};

use super::{AppOrder, Apps, Hit, arrange};
use crate::features::toggles::{EntryState, state};
use crate::{ChangeSet, Snapshot};

#[derive(Clone, Copy, Debug)]
pub struct Shown<'apps> {
    pub hit: Hit<'apps>,
    pub state: EntryState,
}

#[derive(Clone, Copy, Debug)]
pub struct Query<'text> {
    pub category: Option<&'text str>,
    pub search: &'text str,
    pub order: AppOrder,
}

/// The rows of one category, as the page lists them under its heading.
#[derive(Clone, Debug)]
pub struct Group<'apps> {
    pub category: &'apps ToggleSection,
    pub rows: Vec<Shown<'apps>>,
}

/// The category whose apps show: the picked one while it has apps, else
/// `None` for all of them. `ids` are the categories with apps.
pub fn category<'ids>(
    ids: impl IntoIterator<Item = &'ids str>,
    picked: Option<&str>,
) -> Option<&'ids str> {
    let picked = picked?;
    ids.into_iter().find(|id| *id == picked)
}

/// [`rows`] under their categories, in catalog order, each in the chosen
/// order. A category without a row is left out.
pub fn groups<'apps>(
    apps: &'apps Apps,
    query: Query<'_>,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> Vec<Group<'apps>> {
    let mut groups: Vec<Group<'apps>> = apps
        .catalogs
        .iter()
        .flat_map(ToggleCatalog::sections)
        .map(|category| Group {
            category,
            rows: Vec::new(),
        })
        .collect();

    for row in rows(apps, query, snapshot, changes) {
        let group = groups
            .iter_mut()
            .find(|group| std::ptr::eq(group.category, row.hit.category));

        if let Some(group) = group {
            group.rows.push(row);
        }
    }

    groups.retain(|group| !group.rows.is_empty());
    groups
}

pub fn rows<'apps>(
    apps: &'apps Apps,
    query: Query<'_>,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> Vec<Shown<'apps>> {
    let mut hits = apps.filtered(query.category, query.search, snapshot, changes);
    arrange(&mut hits, query.order, snapshot, changes);

    hits.into_iter()
        .map(|hit| Shown {
            hit,
            state: state(hit.catalog, hit.entry, snapshot, changes),
        })
        .collect()
}
