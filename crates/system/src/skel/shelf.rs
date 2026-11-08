#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::path::Path;
use std::time::SystemTime;

use toolbox_catalog::{SkelCatalog, SkelEntry};

/// `files` are full skel paths, for the confirmation that lists what gets replaced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkelRow {
    pub entry: String,
    pub name: String,
    pub files: Vec<String>,
    /// The same files in the home folder, such as `~/.bashrc`, which a
    /// reset replaces and backs up.
    pub targets: Vec<String>,
}

/// What the last reset did. `done` is false until one ran. `failed` holds
/// the names of the entries that could not be reset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub done: bool,
    pub copied: usize,
    pub failed: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Shelf {
    rows: Vec<SkelRow>,
    chosen: BTreeSet<String>,
    resetting: bool,
    outcome: Outcome,
}

impl Shelf {
    /// Offers `rows` from now on. A chosen entry that is gone is forgotten.
    pub fn stock(&mut self, rows: Vec<SkelRow>) {
        self.chosen
            .retain(|entry| rows.iter().any(|row| row.entry == *entry));
        self.rows = rows;
    }

    pub fn rows(&self) -> &[SkelRow] {
        &self.rows
    }

    pub fn choose(&mut self, entry: &str, on: bool) {
        if !on {
            self.chosen.remove(entry);
            return;
        }

        if self.rows.iter().any(|row| row.entry == entry) {
            self.chosen.insert(entry.to_owned());
        }
    }

    pub fn is_chosen(&self, entry: &str) -> bool {
        self.chosen.contains(entry)
    }

    pub fn chosen_count(&self) -> usize {
        self.chosen.len()
    }

    pub fn chosen_targets(&self) -> Vec<String> {
        self.chosen_rows()
            .flat_map(|row| row.targets.iter().cloned())
            .collect()
    }

    pub const fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    pub const fn resetting(&self) -> bool {
        self.resetting
    }

    /// The chosen entries of `catalog`, for the reset that starts with
    /// this call. None while one runs or with nothing chosen.
    pub fn begin_reset(&mut self, catalog: &SkelCatalog) -> Option<Vec<SkelEntry>> {
        let entries: Vec<_> = self
            .chosen_rows()
            .filter_map(|row| catalog.entry(&row.entry))
            .cloned()
            .collect();

        if self.resetting || entries.is_empty() {
            return None;
        }

        self.resetting = true;
        Some(entries)
    }

    pub fn finish_reset(&mut self, outcome: Outcome) {
        self.resetting = false;
        self.outcome = outcome;
        self.chosen.clear();
    }

    fn chosen_rows(&self) -> impl Iterator<Item = &SkelRow> {
        self.rows
            .iter()
            .filter(|row| self.chosen.contains(&row.entry))
    }
}

impl Outcome {
    /// A reset that ended without copying: every entry counts as failed.
    pub fn stopped(entries: &[SkelEntry]) -> Self {
        Self {
            done: true,
            copied: 0,
            failed: entries
                .iter()
                .map(|entry| entry.name().to_owned())
                .collect(),
        }
    }
}

/// Copies `entries` from `skel_root` into `home`. Blocks while it copies
/// whole folders, so it runs off the GUI thread.
pub fn reset(entries: &[SkelEntry], skel_root: &Path, home: &Path) -> Outcome {
    let mut outcome = Outcome {
        done: true,
        ..Outcome::default()
    };

    for entry in entries {
        match super::reset(entry, skel_root, home, SystemTime::now()) {
            Ok(report) => {
                outcome.copied = outcome.copied.saturating_add(report.copied.len());
            }
            Err(error) => {
                tracing::error!("cannot reset {}: {error}", entry.id());
                outcome.failed.push(entry.name().to_owned());
            }
        }
    }

    tracing::info!(
        "reset {} settings files from {}",
        outcome.copied,
        skel_root.display()
    );
    outcome
}

pub fn offered(catalog: &SkelCatalog, skel_root: &Path) -> Vec<SkelRow> {
    super::available(catalog.entries(), skel_root)
        .into_iter()
        .map(|entry| SkelRow {
            entry: entry.id().to_owned(),
            name: entry.name().to_owned(),
            files: entry
                .files()
                .iter()
                .map(|file| skel_root.join(file).display().to_string())
                .collect(),
            targets: entry
                .files()
                .iter()
                .map(|file| format!("~/{}", file.display()))
                .collect(),
        })
        .collect()
}
