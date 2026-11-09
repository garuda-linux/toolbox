//! The logic behind catalog toggle pages (Components, Gaming, the System
//! Settings toggles and the Setup Assistant).
//!
//! An entry's check names one package, unit, group or edit of a user config
//! file, so a click queues exactly one change in that domain and the Review
//! page shows it like any other. A package entry may bring more packages
//! and units along (`also`) and turn other entries off (`conflicts`).

mod also;
mod guards;
pub mod shown;
pub mod summary;
#[cfg(test)]
mod tests;

use toolbox_catalog::{Check, ToggleCatalog, ToggleEntry};
use toolbox_plan::{GroupName, PackageName, Protection};

use crate::domains::{
    Accounts, CurrentUser, EnabledUnits, EnabledUserUnits, Groups, InstalledPackages, Membership,
    Packages, SyncPackages, SystemUnits, UserConfigStates, UserConfigWish, UserConfigs, UserUnits,
};
use crate::features::{kernels, packages, users};
use crate::{ChangeSet, Snapshot, Toggle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntryState {
    /// On in the system, `None` while no probe delivered the state.
    pub system: Option<Toggle>,
    pub queued: Option<Toggle>,
    /// Its requirement holds, counting entries on in the system or in the
    /// queue.
    pub available: bool,
    /// The protected catalog keeps its installed package, so it stays on.
    pub kept: Option<Protection>,
}

impl EntryState {
    /// The check mark: the queued wish, else the system.
    pub const fn shown_on(self) -> bool {
        check_mark(self.queued, self.system)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    Unknown,
    Unmet,
    /// The Users page would refuse the same change of this membership.
    Account(users::Refusal, Membership),
    /// The Kernels page would keep this kernel.
    Kernel(kernels::Refusal, PackageName),
    /// `toolbox_plan::PROTECTED` keeps this package.
    Protected(Protection, PackageName),
    /// Removing the entry's packages would take this protected dependency
    /// along.
    TakesAlong(Protection, PackageName),
}

impl Refusal {
    /// An account refusal keeps the Users page's code, so both pages share
    /// one text.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Unmet => "unmet",
            Self::Account(refusal, _) => refusal.as_str(),
            Self::Kernel(refusal, _) => refusal.as_str(),
            Self::Protected(why, _) | Self::TakesAlong(why, _) => why.as_str(),
        }
    }

    /// The name the Users page's text for the code shows: the group it
    /// is the primary group of, else the user. Empty without an account.
    pub fn name(&self) -> &str {
        match self {
            Self::Unknown | Self::Unmet => "",
            Self::Account(users::Refusal::PrimaryGroup, membership) => membership.group.as_str(),
            Self::Account(_, membership) => membership.user.as_str(),
            Self::Kernel(_, package)
            | Self::Protected(_, package)
            | Self::TakesAlong(_, package) => package.as_str(),
        }
    }
}

pub fn state(
    catalog: &ToggleCatalog,
    entry: &ToggleEntry,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> EntryState {
    let system = system_on(entry.check(), snapshot);
    let kept = match entry.check() {
        Check::Package(name) if system == Some(Toggle::On) => {
            packages::protected::may_remove(name).err()
        }
        Check::Package(_)
        | Check::Service(_)
        | Check::UserService(_)
        | Check::Group(_)
        | Check::UserConfig(_) => None,
    };

    EntryState {
        system,
        queued: queued(entry, snapshot, changes),
        available: is_met(catalog, entry, snapshot, changes),
        kept,
    }
}

/// Whether the entry can be changed on this system.
///
/// A package entry needs its package installed (a `-git` build counts) or
/// offered by the sync databases. A queued change keeps it, so it can be
/// taken back. While the sync databases are unknown every entry counts,
/// other checks always do.
pub fn is_offered(entry: &ToggleEntry, snapshot: &Snapshot, changes: &ChangeSet) -> bool {
    let Check::Package(name) = entry.check() else {
        return true;
    };

    let Some(sync) = snapshot.part::<SyncPackages>() else {
        return true;
    };

    sync.0.contains(name)
        || system_on(entry.check(), snapshot) == Some(Toggle::On)
        || queued(entry, snapshot, changes).is_some()
}

/// Queues entry `id` on or off.
///
/// Turning an entry off also drops the queued "on" of every entry whose
/// requirement no longer holds, since those would fail without it. Turning
/// one on turns the entries it conflicts with off: a queued one is taken
/// back, an installed one is queued for removal.
///
/// # Errors
///
/// [`Refusal`] for an unknown id, an entry whose requirements are off, or
/// a group change the Users page would refuse. Nothing is queued then.
pub fn toggle(
    catalog: &ToggleCatalog,
    id: &str,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) -> Result<(), Refusal> {
    let entry = catalog.entry(id).ok_or(Refusal::Unknown)?;

    if wish.is_on() && !is_met(catalog, entry, snapshot, changes) {
        return Err(Refusal::Unmet);
    }

    // The opposite wish queued means the user takes it back. Without a probe
    // the change set could not tell that the new wish matches the system.
    let takes_back = queued(entry, snapshot, changes) == Some(wish.flipped());

    if !takes_back {
        guards::account_allowed(entry, wish, snapshot, changes)?;
        guards::removal_allowed(entry, wish, snapshot, changes)?;
    }

    if wish.is_on() {
        turn_off_conflicts(catalog, entry, snapshot, changes);
    }

    if takes_back {
        unqueue(catalog, entry, snapshot, changes);
    } else {
        want(catalog, entry, wish, snapshot, changes);
    }

    if !wish.is_on() {
        drop_dependents(catalog, id, snapshot, changes);
    }

    Ok(())
}

fn turn_off_conflicts(
    catalog: &ToggleCatalog,
    entry: &ToggleEntry,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) {
    let rivals: Vec<&ToggleEntry> = entry
        .conflicts()
        .iter()
        .filter_map(|id| catalog.entry(id))
        .filter(|rival| shown_on(rival, snapshot, changes))
        .collect();

    for rival in rivals {
        if let Err(refusal) = toggle(catalog, rival.id(), Toggle::Off, snapshot, changes) {
            tracing::warn!("{} stays on: {}", rival.id(), refusal.as_str());
        }
    }
}

fn is_met(
    catalog: &ToggleCatalog,
    entry: &ToggleEntry,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> bool {
    entry.requires().is_met(|id| {
        catalog
            .entry(id)
            .is_some_and(|required| shown_on(required, snapshot, changes))
    })
}

fn drop_dependents(
    catalog: &ToggleCatalog,
    id: &str,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) {
    let stranded: Vec<&ToggleEntry> = catalog
        .entries()
        .filter(|entry| entry.requires().ids().iter().any(|required| required == id))
        .filter(|entry| queued(entry, snapshot, changes) == Some(Toggle::On))
        .filter(|entry| !is_met(catalog, entry, snapshot, changes))
        .collect();

    for dependent in stranded {
        unqueue(catalog, dependent, snapshot, changes);
        drop_dependents(catalog, dependent.id(), snapshot, changes);
    }
}

fn shown_on(entry: &ToggleEntry, snapshot: &Snapshot, changes: &ChangeSet) -> bool {
    check_mark(
        queued(entry, snapshot, changes),
        system_on(entry.check(), snapshot),
    )
}

/// A `-git` build counts as the package.
fn system_on(check: &Check, snapshot: &Snapshot) -> Option<Toggle> {
    let on = match check {
        Check::Package(name) => snapshot.part::<InstalledPackages>().map(|installed| {
            installed.0.contains(name)
                || git_variant(name).is_some_and(|git| installed.0.contains(&git))
        }),
        Check::Service(unit) => snapshot
            .part::<EnabledUnits>()
            .map(|enabled| enabled.0.contains(unit)),
        Check::UserService(unit) => snapshot
            .part::<EnabledUserUnits>()
            .map(|enabled| enabled.0.contains(unit)),
        Check::Group(group) => {
            let user = snapshot.part::<CurrentUser>()?;

            snapshot
                .part::<Accounts>()
                .map(|accounts| accounts.is_member(&user.0, group))
        }
        Check::UserConfig(edit) => snapshot.part::<UserConfigStates>()?.0.get(edit).copied(),
    };

    on.map(Toggle::from_on)
}

fn queued(entry: &ToggleEntry, snapshot: &Snapshot, changes: &ChangeSet) -> Option<Toggle> {
    let wish = match entry.check() {
        Check::Package(name) => return package_wish(name, changes),
        Check::Service(unit) => changes.wish::<SystemUnits>(unit),
        Check::UserService(unit) => changes.wish::<UserUnits>(unit),
        Check::Group(group) => changes.wish::<Groups>(&membership(group, snapshot)?),
        Check::UserConfig(edit) => {
            return changes.wish::<UserConfigs>(edit).map(|wish| wish.toggle);
        }
    };

    wish.copied()
}

fn package_wish(name: &PackageName, changes: &ChangeSet) -> Option<Toggle> {
    if let Some(wish) = changes.wish::<Packages>(name) {
        return Some(*wish);
    }

    let git = git_variant(name)?;

    changes.wish::<Packages>(&git).copied()
}

fn want(
    catalog: &ToggleCatalog,
    entry: &ToggleEntry,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) {
    match entry.check() {
        Check::Package(name) => {
            want_package(name, wish, snapshot, changes);
            also::want(catalog, entry, wish, snapshot, changes);
        }
        Check::Service(unit) => changes.want::<SystemUnits>(unit.clone(), wish, snapshot),
        Check::UserService(unit) => changes.want::<UserUnits>(unit.clone(), wish, snapshot),
        Check::Group(group) => {
            if let Some(membership) = membership(group, snapshot) {
                changes.want::<Groups>(membership, wish, snapshot);
            }
        }
        Check::UserConfig(edit) => changes.want::<UserConfigs>(
            edit.clone(),
            UserConfigWish::new(wish, entry.restart_user_units().iter().cloned())
                .with_reload(entry.reload()),
            snapshot,
        ),
    }
}

/// Turning off removes the `-git` build when only that one is installed.
fn want_package(name: &PackageName, wish: Toggle, snapshot: &Snapshot, changes: &mut ChangeSet) {
    let installed = snapshot.part::<InstalledPackages>();
    let only_git = git_variant(name).filter(|git| {
        installed.is_some_and(|installed| installed.0.contains(git) && !installed.0.contains(name))
    });

    match only_git {
        Some(git) if !wish.is_on() => changes.want::<Packages>(git, wish, snapshot),
        Some(_) | None => changes.want::<Packages>(name.clone(), wish, snapshot),
    }
}

fn unqueue_package(name: &PackageName, changes: &mut ChangeSet) {
    changes.remove::<Packages>(name);

    if let Some(git) = git_variant(name) {
        changes.remove::<Packages>(&git);
    }
}

fn unqueue(
    catalog: &ToggleCatalog,
    entry: &ToggleEntry,
    snapshot: &Snapshot,
    changes: &mut ChangeSet,
) {
    match entry.check() {
        Check::Package(name) => {
            unqueue_package(name, changes);
            also::unqueue(catalog, entry, snapshot, changes);
        }
        Check::Service(unit) => changes.remove::<SystemUnits>(unit),
        Check::UserService(unit) => changes.remove::<UserUnits>(unit),
        Check::Group(group) => {
            if let Some(membership) = membership(group, snapshot) {
                changes.remove::<Groups>(&membership);
            }
        }
        Check::UserConfig(edit) => changes.remove::<UserConfigs>(edit),
    }
}

/// The app's user in `group`. Without a known user a group entry shows
/// as unknown and a click on it changes nothing.
fn membership(group: &GroupName, snapshot: &Snapshot) -> Option<Membership> {
    let user = snapshot.part::<CurrentUser>()?;

    Some(Membership::new(user.0.clone(), group.clone()))
}

fn git_variant(name: &PackageName) -> Option<PackageName> {
    PackageName::new(format!("{name}-git")).ok()
}

const fn check_mark(queued: Option<Toggle>, system: Option<Toggle>) -> bool {
    match queued {
        Some(wish) => wish.is_on(),
        None => matches!(system, Some(Toggle::On)),
    }
}
