//! Two entries with the same `check` show one state in two places, so one
//! switch flips the other. The catalog refuses the second one instead of
//! showing it.

use std::collections::BTreeSet;

use super::ToggleCatalog;
use crate::Problem;

pub fn check(catalog: &ToggleCatalog) -> Result<(), (String, Problem)> {
    let mut seen = BTreeSet::new();

    for entry in catalog.entries() {
        if !seen.insert(entry.check()) {
            let watched = format!("{:?}", entry.check());

            return Err((entry.id().to_owned(), Problem::DuplicateCheck(watched)));
        }
    }

    Ok(())
}
