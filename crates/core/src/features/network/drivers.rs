//! The Drivers tab: the kernel's network modules, which of them are
//! loaded and which a modprobe file blacklists.
//!
//! The kernel and modprobe read `-` and `_` in a module name as the same
//! character, so every name here is in its `_` spelling, or a loaded
//! module with a `-` in its file name would show as unloaded.

use std::collections::BTreeMap;
use std::iter;

use toolbox_plan::{KernelModule, Plan, RootAction};
pub use toolbox_state::network::drivers::{
    BlacklistEntry, Category, ModuleFacts, NetworkModule, loaded_modules, network_modules,
};

use crate::domains::ModuleBlacklist;
use crate::features::drop_refused;
use crate::features::search::{Needle, Searchable, filter as found};
use crate::{ChangeSet, Snapshot, Toggle};

/// A loaded module that is also blacklisted counts as loaded: the blacklist
/// only keeps the kernel from loading it by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum DriverState {
    Loaded,
    Blacklisted,
    Available,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriverRow {
    pub name: KernelModule,
    pub category: Category,
    pub state: DriverState,
    pub loaded: bool,
    pub own: bool,
    /// Other files that blacklist it. The app shows them and leaves them
    /// alone.
    pub other_files: Vec<String>,
    pub queued: Option<Toggle>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DriverFilter {
    pub search: String,
    pub state: Option<DriverState>,
    pub category: Option<Category>,
}

/// Why a blacklist click was not taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Refusal {
    /// Another modprobe file blacklists the module. The app changes only
    /// its own file.
    OtherFile,
    /// The drivers were not read yet, or could not be.
    Unavailable,
}

impl DriverState {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl Refusal {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

impl Searchable for DriverRow {
    fn texts(&self) -> impl Iterator<Item = &str> {
        iter::once(self.name.as_str())
    }
}

/// Every network module, plus whatever the app's own file blacklists (so
/// it can be taken off again). Loaded ones first, then blacklisted ones,
/// each group by name.
pub fn driver_rows(facts: &ModuleFacts, changes: &ChangeSet) -> Vec<DriverRow> {
    let mut categories: BTreeMap<&KernelModule, Category> = facts
        .blacklists
        .iter()
        .filter(|entry| entry.own)
        .map(|entry| (&entry.module, Category::Other))
        .collect();

    categories.extend(
        facts
            .modules
            .iter()
            .map(|module| (&module.name, module.category)),
    );

    let mut rows: Vec<DriverRow> = categories
        .into_iter()
        .map(|(name, category)| {
            let listed = |own: bool| {
                facts
                    .blacklists
                    .iter()
                    .filter(move |entry| entry.own == own && entry.module == *name)
            };

            let loaded = facts.loaded.contains(name);
            let own = listed(true).next().is_some();
            let other_files: Vec<String> = listed(false).map(|entry| entry.file.clone()).collect();

            DriverRow {
                name: name.clone(),
                category,
                state: if loaded {
                    DriverState::Loaded
                } else if own || !other_files.is_empty() {
                    DriverState::Blacklisted
                } else {
                    DriverState::Available
                },
                loaded,
                own,
                other_files,
                queued: changes.wish::<ModuleBlacklist>(name).copied(),
            }
        })
        .collect();

    rows.sort_by(|left, right| (left.state, &left.name).cmp(&(right.state, &right.name)));
    rows
}

/// The rows `filter` lets through, in their order.
pub fn visible<'rows>(rows: &'rows [DriverRow], filter: &DriverFilter) -> Vec<&'rows DriverRow> {
    // Module names are kept in their `_` spelling, which a search with
    // a `-` would miss.
    let needle = Needle::new(&filter.search.replace('-', "_"));

    found(rows, &needle)
        .filter(|row| filter.state.is_none_or(|state| row.state == state))
        .filter(|row| {
            filter
                .category
                .is_none_or(|category| row.category == category)
        })
        .collect()
}

/// Queues `module` for the app's blacklist file (`on`) or for leaving it,
/// or takes a queued change back.
///
/// # Errors
///
/// [`Refusal::OtherFile`] when only another file blacklists the module:
/// adding it again changes nothing, and taking it off is not the app's to
/// do.
pub fn set_blacklisted(
    module: &KernelModule,
    wish: Toggle,
    facts: &ModuleFacts,
    changes: &mut ChangeSet,
    snapshot: &Snapshot,
) -> Result<(), Refusal> {
    if only_another_file(module, facts) {
        return Err(Refusal::OtherFile);
    }

    changes.want::<ModuleBlacklist>(module.clone(), wish, snapshot);
    Ok(())
}

/// Drops every queued blacklist change the page would refuse: a module
/// only another modprobe file blacklists.
///
/// Returns how many went, `None` until the Network page read the
/// modprobe files.
pub fn recheck_blacklist(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    let facts = snapshot.part::<ModuleFacts>()?;

    Some(drop_refused::<ModuleBlacklist>(
        changes,
        "blacklist change of",
        |module, _, _| only_another_file(module, facts).then_some("another file lists it"),
    ))
}

fn only_another_file(module: &KernelModule, facts: &ModuleFacts) -> bool {
    let mut entries = facts
        .blacklists
        .iter()
        .filter(|entry| entry.module == *module)
        .peekable();

    entries.peek().is_some() && entries.all(|entry| !entry.own)
}

/// Loads the module at once in the Console.
#[must_use = "a plan does nothing until it is run"]
pub fn plan_load(module: &KernelModule) -> Plan {
    Plan::root(RootAction::LoadModule {
        module: module.clone(),
    })
}

/// Unloads the module at once in the Console.
#[must_use = "a plan does nothing until it is run"]
pub fn plan_unload(module: &KernelModule) -> Plan {
    Plan::root(RootAction::UnloadModule {
        module: module.clone(),
    })
}
