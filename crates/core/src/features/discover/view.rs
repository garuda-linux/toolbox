#[cfg(test)]
mod tests;

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

/// The category whose apps show: the picked one while it has apps, else
/// `None` for all of them. `ids` are the categories with apps.
pub fn category<'ids>(
    ids: impl IntoIterator<Item = &'ids str>,
    picked: Option<&str>,
) -> Option<&'ids str> {
    let picked = picked?;
    ids.into_iter().find(|id| *id == picked)
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
