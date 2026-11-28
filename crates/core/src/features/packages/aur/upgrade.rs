//! The AUR part of an upgrade.
//!
//! The outdated AUR packages are built through the split path where the
//! helper accepts the build folder, else paru upgrades them itself. An
//! upgraded package keeps the install reason it has.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use toolbox_plan::{PackageName, PlannedAction, UserAction};
use toolbox_state::aur::order::AurOrder;

use super::{build, takes_split_path};
use crate::Snapshot;
use crate::domains::{BuildOrder, InstalledPackages};

/// The path an AUR upgrade takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpgradePath {
    /// Nothing is outdated, so no AUR step is planned.
    UpToDate,
    /// paru upgrades in the Console, and the user answers its questions.
    Paru,
    /// The split path for these outdated packages, once their orders are read.
    Split(Vec<PackageName>),
}

/// The AUR steps of an upgrade and the path they take. A split path whose
/// orders are not all read takes paru's path instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AurUpgrade {
    pub path: UpgradePath,
    pub actions: Vec<PlannedAction>,
}

impl AurUpgrade {
    /// The whole upgrade: the steps of the system upgrade and the AUR steps.
    ///
    /// The review of a split path comes first, while the person who started
    /// the upgrade is still there, and before anything asks for the password.
    #[must_use = "the steps do nothing until they are planned"]
    pub fn after(self, system: Vec<PlannedAction>) -> Vec<PlannedAction> {
        let mut aur = self.actions.into_iter().peekable();
        let review = aur.next_if(|action| {
            matches!(
                action,
                PlannedAction::User(UserAction::ReviewAur { bases: _ })
            )
        });

        review.into_iter().chain(system).chain(aur).collect()
    }
}

/// The first step of an AUR upgrade: its plan when no read is needed, or the
/// build orders a split path reads before the plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Start {
    Now(AurUpgrade),
    Read(Awaiting),
}

/// A split path whose build orders are still to be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Awaiting {
    names: Vec<PackageName>,
}

impl Awaiting {
    /// The packages whose build orders are read.
    pub fn names(&self) -> &[PackageName] {
        &self.names
    }

    /// The plan with the orders as they were read, paru's when one did not
    /// come back.
    #[must_use = "the steps do nothing until they are planned"]
    pub fn plan(self, orders: Option<&[BuildOrder]>) -> AurUpgrade {
        plan_upgrade(UpgradePath::Split(self.names), orders)
    }
}

/// The path of an upgrade of the outdated AUR packages.
///
/// `outdated` is `None` while that list is not read. The split path needs
/// the helper's accepted route and the installed packages, as the install
/// path does.
pub fn upgrade_path(snapshot: &Snapshot, outdated: Option<&[PackageName]>) -> UpgradePath {
    let split_ready = takes_split_path(snapshot) && snapshot.part::<InstalledPackages>().is_some();

    match outdated {
        Some(names) if split_ready && names.is_empty() => UpgradePath::UpToDate,
        Some(names) if split_ready => UpgradePath::Split(names.to_vec()),
        None | Some(_) => UpgradePath::Paru,
    }
}

/// Where an upgrade of the outdated AUR packages starts, see [`Start`].
#[must_use = "the upgrade does nothing until it is started"]
pub fn start(snapshot: &Snapshot, outdated: Option<&[PackageName]>) -> Start {
    match upgrade_path(snapshot, outdated) {
        UpgradePath::Split(names) => Start::Read(Awaiting { names }),
        path @ (UpgradePath::UpToDate | UpgradePath::Paru) => Start::Now(plan_upgrade(path, None)),
    }
}

/// The AUR steps of `path`, paru's upgrade where the split path cannot take
/// them.
///
/// `orders` holds the build orders read for a split path, and `None` when
/// one of them could not be read.
pub fn plan_upgrade(path: UpgradePath, orders: Option<&[BuildOrder]>) -> AurUpgrade {
    match path {
        UpgradePath::UpToDate => AurUpgrade {
            path: UpgradePath::UpToDate,
            actions: Vec::new(),
        },
        UpgradePath::Paru => paru(),
        UpgradePath::Split(names) => orders
            .and_then(|orders| split_steps(&names, orders))
            .map_or_else(paru, |actions| AurUpgrade {
                path: UpgradePath::Split(names),
                actions,
            }),
    }
}

fn paru() -> AurUpgrade {
    AurUpgrade {
        path: UpgradePath::Paru,
        actions: vec![PlannedAction::User(UserAction::UpgradeAur)],
    }
}

/// The builder's steps for `names`, or `None` when the split path cannot take
/// them.
///
/// The orders must share one run, each name needs an order and each name must
/// be installed. The builder gives an installed package `Keep`, so an upgraded
/// package keeps its reason.
fn split_steps(names: &[PackageName], orders: &[BuildOrder]) -> Option<Vec<PlannedAction>> {
    let run = &orders.first()?.run;
    if orders.iter().any(|order| order.run != *run) {
        return None;
    }

    let installed: BTreeSet<PackageName> = orders
        .iter()
        .flat_map(|order| order.installed.iter().cloned())
        .collect();

    if !names.iter().all(|name| installed.contains(name)) {
        return None;
    }

    let relevant: Vec<AurOrder> = orders
        .iter()
        .filter(|order| names.iter().any(|name| build::mentions(&order.order, name)))
        .map(|order| order.order.clone())
        .collect();

    build::plan(names, &relevant, run, &installed)
        .ok()
        .map(|planned| planned.actions)
}
