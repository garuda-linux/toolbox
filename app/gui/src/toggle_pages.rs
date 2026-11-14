//! The toggle pages' clicks and the last refused one of each page.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;

use toolbox_core::features::toggles::{self, Refusal};
use toolbox_core::{ChangeSet, Snapshot};

use crate::catalogs::{CatalogId, Catalogs};
use crate::command::AppCommand;
use crate::page_service::Handled;
use crate::refused::{Refused, queue_or_keep};

pub type ToggleRefusal = Refused<Refusal>;

#[derive(Debug, Default)]
pub struct TogglePages {
    /// Shared with App, which loads them once at the start.
    catalogs: Arc<Catalogs>,
    refusals: HashMap<CatalogId, ToggleRefusal>,
}

impl TogglePages {
    pub fn new(catalogs: Arc<Catalogs>) -> Self {
        Self {
            catalogs,
            refusals: HashMap::new(),
        }
    }

    pub fn refusal(&self, catalog: CatalogId) -> Option<&ToggleRefusal> {
        self.refusals.get(&catalog)
    }

    /// A taken click clears its page's refusal, a refused one names why.
    pub fn set_refusal(&mut self, catalog: CatalogId, refusal: Option<ToggleRefusal>) {
        match refusal {
            Some(refusal) => drop(self.refusals.insert(catalog, refusal)),
            None => drop(self.refusals.remove(&catalog)),
        }
    }

    pub fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        if let AppCommand::DismissRefusal(catalog) = command {
            self.set_refusal(*catalog, None);
            return Handled::taken();
        }

        let AppCommand::ToggleEntry {
            catalog,
            entry,
            wish,
        } = command
        else {
            return Handled::default();
        };

        let Ok(toggle_catalog) = &self.catalogs.get(*catalog).catalog else {
            return Handled::taken();
        };

        let mut refusal = self.refusals.remove(catalog);
        let handled = queue_or_keep(&mut refusal, changes, |changes| {
            toggles::toggle(toggle_catalog, entry, *wish, snapshot, changes)
        });

        self.set_refusal(*catalog, refusal);
        handled
    }
}
