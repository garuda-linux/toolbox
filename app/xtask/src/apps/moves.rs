//! An app whose first package changed since the shipped catalog. Its
//! switch would check and remove another package, and a queued change
//! saved under the old id no longer matches, so a write needs consent.

use std::collections::BTreeMap;

use toolbox_catalog::{Check, ToggleCatalog};

use super::{Category, Lists};

/// The first package per app, by category id and title.
pub type Firsts = BTreeMap<(String, String), String>;

pub fn firsts(shipped: &[(Category, ToggleCatalog)]) -> Firsts {
    let mut firsts = Firsts::new();

    for (category, catalog) in shipped {
        for entry in catalog.entries() {
            if let Check::Package(name) = entry.check() {
                firsts.insert(
                    (category.id.to_owned(), entry.title().to_owned()),
                    name.as_str().to_owned(),
                );
            }
        }
    }

    firsts
}

/// One line per app the wiki now lists with another first package.
pub fn moves(shipped: &Firsts, lists: &Lists) -> Vec<String> {
    let mut lines = Vec::new();

    for (category, apps) in &lists.lists {
        for (_, app) in apps {
            let key = (category.id.to_owned(), app.title.clone());
            let (Some(before), Some(after)) = (shipped.get(&key), app.packages.first()) else {
                continue;
            };

            if before != after {
                lines.push(format!(
                    "first package moved: {}: {:?} from {before} to {after}",
                    category.id, app.title
                ));
            }
        }
    }

    lines
}

/// A write with moves fails unless they were accepted. A dry run only
/// lists them.
///
/// # Errors
///
/// The moves and the flag that accepts them.
pub fn gate(moves: &[String], write: bool, accept: bool) -> Result<(), Vec<String>> {
    if !write || accept || moves.is_empty() {
        return Ok(());
    }

    let mut problems = moves.to_vec();

    problems.push("nothing written: check the moves, then pass --accept-moves".to_owned());
    Err(problems)
}
