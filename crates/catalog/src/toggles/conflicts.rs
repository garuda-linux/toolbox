//! The rules for `conflicts` across a whole toggle catalog file.
//!
//! ```toml
//! conflicts = ["libreoffice-still"]
//! ```
//!
//! Turning the entry on turns the named entries off. One direction is
//! enough for the catalog to load, an exclusive pair names each other.

use std::collections::BTreeSet;

use super::{ToggleCatalog, ToggleEntry};
use crate::{EntryProblem, Problem};

/// The entry's `conflicts` as written, `None` when the field is missing.
pub fn list(raw: Option<Vec<String>>) -> Result<Vec<String>, Problem> {
    let Some(ids) = raw else {
        return Ok(Vec::new());
    };

    if ids.is_empty() {
        return Err(Problem::EmptyConflicts);
    }

    let mut seen = BTreeSet::new();

    if let Some(twice) = ids.iter().find(|id| !seen.insert(id.as_str())) {
        return Err(Problem::ConflictRepeats(twice.clone()));
    }

    Ok(ids)
}

/// The first entry, in file order, whose `conflicts` names itself, an id
/// that is no entry of the file, or an id it also requires.
pub fn check(catalog: &ToggleCatalog) -> Result<(), EntryProblem> {
    let known: BTreeSet<&str> = catalog.entries().map(ToggleEntry::id).collect();

    for entry in catalog.entries() {
        for other in entry.conflicts() {
            let problem = if other == entry.id() {
                Problem::ConflictsItself
            } else if entry.requires().ids().contains(other) {
                Problem::ConflictsRequirement(other.clone())
            } else if known.contains(other.as_str()) {
                continue;
            } else {
                Problem::UnknownConflict(other.clone())
            };

            return Err(EntryProblem::new(entry.id(), problem));
        }
    }

    Ok(())
}
