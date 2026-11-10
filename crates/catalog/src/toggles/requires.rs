//! What an entry depends on, and the rules for `requires` across a whole
//! toggle catalog file.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::{ToggleCatalog, ToggleEntry};
use crate::{EntryProblem, Problem};

type Graph<'a> = BTreeMap<&'a str, &'a [String]>;

/// The entries an entry depends on. `{ all = [..] }` needs every one of
/// them on, `{ any = [..] }` at least one, for example Docker or Podman.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum Requires {
    #[default]
    #[serde(skip)]
    Nothing,
    All(Vec<String>),
    Any(Vec<String>),
}

impl Requires {
    pub fn ids(&self) -> &[String] {
        match self {
            Self::Nothing => &[],
            Self::All(ids) | Self::Any(ids) => ids,
        }
    }

    pub fn is_met<F>(&self, mut on: F) -> bool
    where
        F: FnMut(&str) -> bool,
    {
        match self {
            Self::Nothing => true,
            Self::All(ids) => ids.iter().all(|id| on(id)),
            Self::Any(ids) => ids.iter().any(|id| on(id)),
        }
    }
}

/// The first entry, in file order, whose `requires` is an empty list, names
/// a missing entry or itself, or starts a cycle.
pub fn check(catalog: &ToggleCatalog) -> Result<(), EntryProblem> {
    let graph: Graph<'_> = catalog
        .entries()
        .map(|entry| (entry.id(), entry.requires().ids()))
        .collect();

    for entry in catalog.entries() {
        check_targets(entry, &graph).map_err(|problem| EntryProblem::new(entry.id(), problem))?;
    }

    let mut done = BTreeSet::new();

    for entry in catalog.entries() {
        let mut path = Vec::new();

        if let Some((start, cycle)) = find_cycle(entry.id(), &graph, &mut path, &mut done) {
            return Err(EntryProblem::new(start, Problem::RequirementCycle(cycle)));
        }
    }

    Ok(())
}

fn check_targets(entry: &ToggleEntry, graph: &Graph<'_>) -> Result<(), Problem> {
    if matches!(entry.requires(), Requires::All(ids) | Requires::Any(ids) if ids.is_empty()) {
        return Err(Problem::EmptyRequirement);
    }

    let mut seen = BTreeSet::new();

    for required in entry.requires().ids() {
        if !seen.insert(required.as_str()) {
            return Err(Problem::RequirementRepeats(required.clone()));
        }

        if required == entry.id() {
            return Err(Problem::RequiresItself);
        }

        if !graph.contains_key(required.as_str()) {
            return Err(Problem::UnknownRequirement(required.clone()));
        }
    }

    Ok(())
}

/// Depth-first search from `id`. Returns the entry that closes the cycle
/// and the ids along it, starting there. `done` holds entries already
/// proven cycle free, so shared requirements are walked once.
fn find_cycle<'a>(
    id: &'a str,
    graph: &Graph<'a>,
    path: &mut Vec<&'a str>,
    done: &mut BTreeSet<&'a str>,
) -> Option<(&'a str, Vec<String>)> {
    if done.contains(id) {
        return None;
    }

    if let Some(start) = path.iter().position(|step| *step == id) {
        let cycle = path.iter().skip(start).map(|step| (*step).to_owned());

        return Some((id, cycle.collect()));
    }

    path.push(id);

    for required in graph.get(id).copied().unwrap_or_default() {
        if let Some(found) = find_cycle(required, graph, path, done) {
            return Some(found);
        }
    }

    path.pop();
    done.insert(id);

    None
}
