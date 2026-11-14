//! The toggle pages' and Discover's clicks, the choices of System Tweaks
//! and the last refused click of each page.

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;

use toolbox_core::features::toggles::{self, Refusal};
use toolbox_core::features::tweaks::{self, Tweak};
use toolbox_core::{ChangeSet, Snapshot, Toggle};
use toolbox_system::Host;

use crate::catalogs::{CatalogId, Catalogs};
use crate::command::{AppCommand, DiscoverCommand, TweakCommand};
use crate::page::PageId;
use crate::page_service::Handled;
use crate::refused::{Refused, queue, queue_or_keep};

pub type ToggleRefusal = Refused<Refusal>;

#[derive(Debug, Default)]
pub struct TogglePages {
    /// Shared with App, which loads them once at the start.
    catalogs: Arc<Catalogs>,
    refusals: HashMap<CatalogId, ToggleRefusal>,
    /// Discover's, whose categories are one page.
    apps: Option<ToggleRefusal>,
}

impl TogglePages {
    pub fn new(catalogs: Arc<Catalogs>) -> Self {
        Self {
            catalogs,
            refusals: HashMap::new(),
            apps: None,
        }
    }

    pub fn refusal(&self, catalog: CatalogId) -> Option<&ToggleRefusal> {
        self.refusals.get(&catalog)
    }

    pub const fn app_refusal(&self) -> Option<&ToggleRefusal> {
        self.apps.as_ref()
    }

    /// A taken click clears its page's refusal, a refused one names why.
    pub fn set_refusal(&mut self, catalog: CatalogId, refusal: Option<ToggleRefusal>) {
        match refusal {
            Some(refusal) => drop(self.refusals.insert(catalog, refusal)),
            None => drop(self.refusals.remove(&catalog)),
        }
    }

    /// Whether `page` is a toggle page or Discover, whose refusal goes.
    pub fn dismiss_refusal(&mut self, page: PageId) -> bool {
        if page == PageId::Discover {
            self.apps = None;
            return true;
        }

        let catalog = CatalogId::of_page(page);
        if let Some(catalog) = catalog {
            self.set_refusal(catalog, None);
        }

        catalog.is_some()
    }

    pub fn handle(
        &mut self,
        command: &AppCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        if let AppCommand::Discover(command) = command {
            return self.discover(command, changes, snapshot);
        }

        if let AppCommand::Tweak(command) = command {
            return tweak(command, changes, snapshot);
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

    fn discover(
        &mut self,
        command: &DiscoverCommand,
        changes: &ChangeSet,
        snapshot: &Snapshot,
    ) -> Handled {
        match command {
            DiscoverCommand::Toggle { app, wish } => {
                let apps = &self.catalogs.apps.apps;
                queue_or_keep(&mut self.apps, changes, |changes| {
                    apps.toggle(app, *wish, snapshot, changes)
                })
            }
            DiscoverCommand::Reveal(_) => Handled::default(),
        }
    }
}

/// A wish the system already fulfils is not queued, as on every page.
fn tweak(command: &TweakCommand, changes: &ChangeSet, snapshot: &Snapshot) -> Handled {
    let tweak = match command {
        // The probe names the account. `USER` names the wrong one after
        // `sudo -u`, so it is no stand-in before that.
        TweakCommand::Shell(shell) => {
            let Some(user) = snapshot.part::<Host>().and_then(|host| host.user.clone()) else {
                return Handled::taken();
            };

            Tweak::Shell(user, shell.clone())
        }
        TweakCommand::Dns(provider) => Tweak::Dns(*provider),
        TweakCommand::Wifi(backend) => Tweak::Wifi(*backend),
        TweakCommand::Hblock(on) => Tweak::Hblock(Toggle::from_on(*on)),
        TweakCommand::Flag(flag, on) => Tweak::Flag(*flag, Toggle::from_on(*on)),
    };

    queue(changes, |changes| tweaks::queue(changes, tweak, snapshot))
}
