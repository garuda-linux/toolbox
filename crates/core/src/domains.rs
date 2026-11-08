//! The domains the core ships. A feature adds its own next to these.

mod accounts;
mod aur;
mod boot;
mod dkms;
mod garuda_update;
mod groups;
mod hblock;
mod locales;
mod masks;
mod modules;
mod network;
mod packages;
mod shells;
mod units;
mod user_configs;
mod users;

pub use accounts::{AccountLockFlags, Accounts, CurrentUser, Group, User};
pub use aur::AurPackages;
pub use boot::{
    BootLoader, CurrentGrub, GrubDefaults, KernelCmdline, PlymouthState, PlymouthThemes,
};
pub use dkms::DkmsBuilds;
pub use garuda_update::{GarudaUpdateConfig, GarudaUpdateFlags};
pub use groups::{Groups, Membership};
pub use hblock::{Hblock, HblockState};
pub use locales::{CurrentSystemLocale, EnabledLocales, Locales, SystemLocales};
pub use masks::{SystemUnitMasks, UserUnitMasks};
pub use modules::{ModuleBlacklist, OwnBlacklist};
pub use network::{CurrentDns, CurrentWifiBackend, Dns, WifiBackends};
pub use packages::{InstalledPackages, Packages, SyncPackages};
pub use shells::{AvailableShells, CurrentShells, LoginShells};
use toolbox_plan::{PackageName, PlannedAction, RootAction, TargetList};
use toolbox_state::Toggle;
pub use units::{EnabledUnits, EnabledUserUnits, SystemUnits, UserUnits};
pub use user_configs::{UserConfigStates, UserConfigWish, UserConfigs};
pub use users::{AccountLocks, FullNames, HomeFolders, NewGroups, RemovedGroups, RemovedUsers};

use crate::domain::{Stage, Staged};

#[cfg(test)]
mod tests;

/// Every domain the app knows, for restoring saved changes.
///
/// # Errors
///
/// When two of them share an id.
pub fn all() -> Result<crate::Domains, crate::DuplicateDomain> {
    crate::Domains::default()
        .with::<Packages>()?
        .with::<AurPackages>()?
        .with::<Groups>()?
        .with::<SystemUnits>()?
        .with::<UserUnits>()?
        .with::<SystemUnitMasks>()?
        .with::<UserUnitMasks>()?
        .with::<LoginShells>()?
        .with::<Dns>()?
        .with::<Hblock>()?
        .with::<WifiBackends>()?
        .with::<GarudaUpdateFlags>()?
        .with::<UserConfigs>()?
        .with::<DkmsBuilds>()?
        .with::<Locales>()?
        .with::<SystemLocales>()?
        .with::<NewGroups>()?
        .with::<AccountLocks>()?
        .with::<FullNames>()?
        .with::<HomeFolders>()?
        .with::<RemovedUsers>()?
        .with::<RemovedGroups>()?
        .with::<ModuleBlacklist>()?
        .with::<GrubDefaults>()?
        .with::<PlymouthThemes>()?
        .with::<crate::features::maintenance::Operations>()
}

/// `pacman -S --needed` of one package.
pub(crate) fn install(package: &PackageName) -> RootAction {
    RootAction::InstallPackages {
        packages: TargetList::one(package.clone()),
    }
}

/// One root action per change, each at `stage`.
fn each<I, W>(
    stage: Stage,
    changes: &[(&I, &W)],
    action: impl Fn(&I, &W) -> RootAction,
) -> Vec<Staged> {
    changes
        .iter()
        .map(|(item, wish)| Staged::at(stage, PlannedAction::Root(action(item, wish))))
        .collect()
}

/// One action for all of `items` at `stage`, none for no items.
fn batch<T>(
    stage: Stage,
    items: Vec<T>,
    action: impl FnOnce(TargetList<T>) -> PlannedAction,
) -> Option<Staged>
where
    T: Ord,
{
    TargetList::new(items)
        .ok()
        .map(|targets| Staged::at(stage, action(targets)))
}

/// Items to switch on and items to switch off, for toggle domains.
fn split<T>(changes: &[(&T, &Toggle)]) -> (Vec<T>, Vec<T>)
where
    T: Clone,
{
    let (on, off): (Vec<_>, Vec<_>) = changes.iter().partition(|(_, wish)| wish.is_on());
    let items = |part: Vec<&(&T, &Toggle)>| -> Vec<T> {
        part.into_iter().map(|(item, _)| (*item).clone()).collect()
    };

    (items(on), items(off))
}

/// The batches a toggle domain planned, in the order its `plan` lists
/// them, without the empty ones.
fn staged(parts: impl IntoIterator<Item = Option<Staged>>) -> Vec<Staged> {
    parts.into_iter().flatten().collect()
}
