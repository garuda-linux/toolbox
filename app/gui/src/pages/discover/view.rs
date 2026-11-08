#[cfg(test)]
mod tests;

use toolbox_core::features::discover::{AppOrder, Apps, Category, Hit, arrange};
use toolbox_core::features::toggles::{EntryState, state};
use toolbox_core::{ChangeSet, Snapshot};

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

/// The category whose apps show: the picked one while it has apps, else
/// `None` for all of them.
pub fn category<'apps>(categories: &[Category<'apps>], picked: Option<&str>) -> Option<&'apps str> {
    let picked = picked?;
    categories
        .iter()
        .map(|category| category.section.id())
        .find(|id| *id == picked)
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
