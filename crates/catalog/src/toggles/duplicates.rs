//! Two entries with the same `check` show one state in two places, so one
//! switch flips the other. The catalog refuses the second one instead of
//! showing it.

use std::collections::BTreeSet;

use super::ToggleCatalog;
use crate::{EntryProblem, Problem};

pub fn check(catalog: &ToggleCatalog) -> Result<(), EntryProblem> {
    let mut seen = BTreeSet::new();

    for entry in catalog.entries() {
        if !seen.insert(entry.check()) {
            let watched = entry.check().to_string();

            return Err(EntryProblem::new(
                entry.id(),
                Problem::DuplicateCheck(watched),
            ));
        }
    }

    Ok(())
}
