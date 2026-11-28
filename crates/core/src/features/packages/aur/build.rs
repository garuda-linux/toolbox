//! The plan of an AUR install through paru: the repository packages first,
//! then each pkgbase built as the user and installed by the helper.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use toolbox_plan::{
    BuildRun, InstallReason, PackageName, PlannedAction, RootAction, TargetList, UserAction,
    protection,
};
use toolbox_state::aur::order::{AurOrder, Role};
use toolbox_text::OutsideText;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildPlan {
    pub actions: Vec<PlannedAction>,
    pub bases: Vec<PackageName>,
}

/// What stops a plan. Paru's `MISSING` names are outside text. A queued name
/// that no order mentions is one the user asked for.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("paru names no package for some queued names")]
pub struct Missing {
    pub paru: Vec<OutsideText>,
    pub unordered: Vec<PackageName>,
}

/// The lines of all orders, each name once.
struct Merged {
    repo: Vec<(PackageName, Role)>,
    aur: Vec<(PackageName, Role)>,
    pkgbases: Vec<(PackageName, PackageName)>,
}

impl Merged {
    fn of(orders: &[AurOrder]) -> Self {
        let mut merged = Self {
            repo: Vec::new(),
            aur: Vec::new(),
            pkgbases: Vec::new(),
        };

        for order in orders {
            for line in &order.repo {
                merge(&mut merged.repo, &line.name, line.role);
            }

            for build in &order.builds {
                merge(&mut merged.aur, &build.name, build.role);
                if !merged.pkgbases.iter().any(|(name, _)| *name == build.name) {
                    merged
                        .pkgbases
                        .push((build.name.clone(), build.base.clone()));
                }
            }
        }

        merged
    }

    /// The pkgbases in the order their first package appears.
    fn bases(&self) -> Vec<PackageName> {
        let mut bases: Vec<PackageName> = Vec::new();
        for (name, _) in &self.aur {
            if let Some(base) = base_of(&self.pkgbases, name)
                && !bases.contains(base)
            {
                bases.push(base.clone());
            }
        }

        bases
    }

    fn actions(
        &self,
        bases: &[PackageName],
        run: &BuildRun,
        installed: &BTreeSet<PackageName>,
    ) -> Vec<PlannedAction> {
        let mut actions = Vec::new();

        if let Ok(bases) = TargetList::new(bases.iter().cloned()) {
            actions.push(PlannedAction::User(UserAction::ReviewAur { bases }));
        }

        let targets = names_with(&self.repo, |role| role == Role::Target);
        let dependencies = names_with(&self.repo, |role| role != Role::Target);
        if let Ok(packages) = TargetList::new(targets) {
            actions.push(PlannedAction::Root(RootAction::InstallPackages {
                packages,
            }));
        }

        if let Ok(packages) = TargetList::new(dependencies) {
            actions.push(PlannedAction::Root(RootAction::InstallDependencies {
                packages,
            }));
        }

        for base in bases {
            actions.push(PlannedAction::User(UserAction::BuildAur {
                base: base.clone(),
                run: run.clone(),
            }));

            let built: Vec<(PackageName, InstallReason)> = self
                .aur
                .iter()
                .filter(|(name, _)| base_of(&self.pkgbases, name) == Some(base))
                .map(|(name, role)| (name.clone(), reason_of(name, *role, installed)))
                .collect();

            actions.extend(installs(run, &built));
        }

        if let Ok(packages) = TargetList::new(self.make_only(installed)) {
            actions.push(PlannedAction::Root(RootAction::RemoveMakeDependencies {
                packages,
            }));
        }

        actions
    }

    /// The MAKE names that no installed package keeps. A protected name stays.
    fn make_only(&self, installed: &BTreeSet<PackageName>) -> Vec<PackageName> {
        self.repo
            .iter()
            .chain(self.aur.iter())
            .filter(|(name, role)| {
                *role == Role::Make && !installed.contains(name) && protection(name).is_none()
            })
            .map(|(name, _)| name.clone())
            .collect::<BTreeSet<PackageName>>()
            .into_iter()
            .collect()
    }
}

/// What the build of one order does, for its review row: the pkgbases in
/// build order and the repository packages it installs as dependencies.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outline {
    pub bases: Vec<PackageName>,
    pub dependencies: Vec<PackageName>,
}

pub fn outline(order: &AurOrder) -> Outline {
    let merged = Merged::of(std::slice::from_ref(order));

    Outline {
        bases: merged.bases(),
        dependencies: names_with(&merged.repo, |role| role != Role::Target),
    }
}

/// Merges the orders of the queued packages into one plan. A repeated line
/// keeps the position of its first occurrence and the strongest role.
///
/// # Errors
///
/// [`Missing`] when an order has a `MISSING` line or a queued name is in no
/// order. No plan is made then.
pub fn plan(
    queued: &[PackageName],
    orders: &[AurOrder],
    run: &BuildRun,
    installed: &BTreeSet<PackageName>,
) -> Result<BuildPlan, Missing> {
    check(queued, orders)?;

    let merged = Merged::of(orders);
    let bases = merged.bases();
    let actions = merged.actions(&bases, run, installed);

    Ok(BuildPlan { actions, bases })
}

/// # Errors
///
/// [`Missing`] for each `MISSING` line and each queued name in no order.
fn check(queued: &[PackageName], orders: &[AurOrder]) -> Result<(), Missing> {
    let mut paru: Vec<OutsideText> = Vec::new();
    for line in orders.iter().flat_map(|order| order.missing.iter()) {
        if !paru.contains(line) {
            paru.push(line.clone());
        }
    }

    let unordered: Vec<PackageName> = queued
        .iter()
        .filter(|name| !orders.iter().any(|order| mentions(order, name)))
        .cloned()
        .collect();

    if paru.is_empty() && unordered.is_empty() {
        Ok(())
    } else {
        Err(Missing { paru, unordered })
    }
}

pub(crate) fn mentions(order: &AurOrder, name: &PackageName) -> bool {
    order.repo.iter().any(|line| line.name == *name)
        || order.builds.iter().any(|build| build.name == *name)
}

/// The pkgbase of a built package, the first order that names it decides.
fn base_of<'list>(
    pkgbases: &'list [(PackageName, PackageName)],
    name: &PackageName,
) -> Option<&'list PackageName> {
    pkgbases
        .iter()
        .find(|(known, _)| known == name)
        .map(|(_, base)| base)
}

fn merge(lines: &mut Vec<(PackageName, Role)>, name: &PackageName, role: Role) {
    match lines.iter_mut().find(|(known, _)| known == name) {
        Some((_, kept)) => *kept = stronger(*kept, role),
        None => lines.push((name.clone(), role)),
    }
}

/// TARGET over DEP over MAKE.
const fn stronger(left: Role, right: Role) -> Role {
    match (left, right) {
        (Role::Target, _) | (_, Role::Target) => Role::Target,
        (Role::Dependency, _) | (_, Role::Dependency) => Role::Dependency,
        (Role::Make, Role::Make) => Role::Make,
    }
}

fn names_with(lines: &[(PackageName, Role)], wanted: impl Fn(Role) -> bool) -> Vec<PackageName> {
    lines
        .iter()
        .filter(|(_, role)| wanted(*role))
        .map(|(name, _)| name.clone())
        .collect()
}

/// A package that is installed keeps its reason. A queued target is
/// explicit, any other package is a dependency.
/// The install of one base's packages in one `pacman -U`, since pacman
/// moves a file between them only inside one transaction.
///
/// One reason for all is set by the install. With several it sets none: an
/// installed package keeps its own, and the new dependencies are marked.
fn installs(run: &BuildRun, built: &[(PackageName, InstallReason)]) -> Vec<PlannedAction> {
    let Ok(packages) = TargetList::new(built.iter().map(|(name, _)| name.clone())) else {
        return Vec::new();
    };

    let shared = built
        .first()
        .map(|(_, reason)| *reason)
        .filter(|first| built.iter().all(|(_, reason)| reason == first));

    let mut actions = vec![PlannedAction::Root(RootAction::InstallBuilt {
        run: run.clone(),
        packages,
        reason: shared.unwrap_or(InstallReason::Keep),
    })];

    let dependencies = built
        .iter()
        .filter(|(_, reason)| shared.is_none() && *reason == InstallReason::Dependency)
        .map(|(name, _)| name.clone());

    if let Ok(marked) = TargetList::new(dependencies) {
        actions.push(PlannedAction::Root(RootAction::MarkDependencies {
            packages: marked,
        }));
    }

    actions
}

fn reason_of(name: &PackageName, role: Role, installed: &BTreeSet<PackageName>) -> InstallReason {
    if installed.contains(name) {
        return InstallReason::Keep;
    }

    match role {
        Role::Target => InstallReason::Explicit,
        Role::Dependency | Role::Make => InstallReason::Dependency,
    }
}
