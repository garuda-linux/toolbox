//! The logic behind catalog toggle pages (Components, Gaming, the System
//! Settings toggles and the Setup Assistant).
//!
//! An entry's check names one package, unit, group or edit of a user config
//! file, so a click queues exactly one change. A package entry may bring more
//! packages and units along (`also`) and turn other entries off (`conflicts`).

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
use crate::features::{RefusalText as _, Refused, kernels, packages, users};
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

/// What the refusal names is its [`Refused::subject`]: the kernel, the
/// protected package, or the account of the membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Unknown,
    Unmet,
    /// The Users page would refuse the same change of the membership.
    Account(users::Refusal),
    /// The Kernels page would keep the kernel.
    Kernel(kernels::Refusal),
    /// `toolbox_plan::PROTECTED` keeps the package.
    Protected(Protection),
    /// Removing the entry's packages would take this protected dependency
    /// along.
    TakesAlong(Protection),
}

/// An account or kernel refusal keeps the other page's code, so both pages
/// share one text.
impl From<Refusal> for &'static str {
    fn from(refusal: Refusal) -> Self {
        match refusal {
            Refusal::Unknown => "unknown",
            Refusal::Unmet => "unmet",
            Refusal::Account(refusal) => refusal.code(),
            Refusal::Kernel(refusal) => refusal.code(),
            Refusal::Protected(why) | Refusal::TakesAlong(why) => why.as_str(),
        }
    }
}

impl Refusal {
    /// The Users page's text for a membership names the group it is the
    /// primary group of, else the user.
    pub(super) fn of_membership(refusal: users::Refusal, membership: &Membership) -> Refused<Self> {
        let name = if refusal == users::Refusal::PrimaryGroup {
            membership.group.as_str()
        } else {
            membership.user.as_str()
        };

        Refused::new(Self::Account(refusal), name.to_owned())
    }

    pub(super) fn of_package(self, package: &PackageName) -> Refused<Self> {
        Refused::new(self, package.as_str().to_owned())
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
) -> Result<(), Refused<Refusal>> {
    let entry = catalog
        .entry(id)
        .ok_or_else(|| Refused::new(Refusal::Unknown, id.to_owned()))?;

    if wish.is_on() && !is_met(catalog, entry, snapshot, changes) {
        return Err(Refused::new(Refusal::Unmet, id.to_owned()));
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
            tracing::warn!("{} stays on: {}", rival.id(), refusal.refusal.code());
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

fn want_package(name: &PackageName, wish: Toggle, snapshot: &Snapshot, changes: &mut ChangeSet) {
    let package = if wish.is_on() {
        name.clone()
    } else {
        removed_package(name, snapshot)
    };

    changes.want::<Packages>(package, wish, snapshot);
}

/// What turning `name` off removes: the `-git` build when only that one
/// is installed.
fn removed_package(name: &PackageName, snapshot: &Snapshot) -> PackageName {
    let installed = snapshot.part::<InstalledPackages>();

    git_variant(name)
        .filter(|git| {
            installed
                .is_some_and(|installed| installed.0.contains(git) && !installed.0.contains(name))
        })
        .unwrap_or_else(|| name.clone())
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
