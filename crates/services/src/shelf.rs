//! The Maintenance page's choice of settings files to reset from
//! `/etc/skel`, and whether a reset runs. Page state, so it lives here and
//! not in `toolbox-system`, which only reads and copies the files.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use toolbox_catalog::{SkelCatalog, SkelEntry};
use toolbox_plan::Toggle;
use toolbox_system::skel::shelf::{Outcome, SkelRow};

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

    pub fn choose(&mut self, entry: &str, wish: Toggle) {
        let on = wish.is_on();
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
