//! Orders come from the Lima captures, and the cases that need another
//! order edit a parsed capture in the test, which names the edit.

mod captures;
mod merge;
mod missing;
mod octopi;

use std::collections::BTreeSet;

use toolbox_plan::{
    BuildRun, InstallReason, PackageName, PlannedAction, RootAction, TargetList, UserAction,
};
use toolbox_state::aur::order::{AurOrder, Role, parse_order};
use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;

use super::{BuildPlan, Missing, plan};
use crate::test_support::package;

/// `paru -Po --aur octopi-git` as parsed.
fn octopi() -> AurOrder {
    capture(OCTOPI_GIT_STDOUT)
}

fn capture(text: &str) -> AurOrder {
    parse_order(text).unwrap()
}

/// The octopi order without its builds and with `name` as a repository line
/// of `role`. Paru prints a repository package that was asked for as TARGET.
fn repo_as(name: &str, role: Role) -> AurOrder {
    let mut order = octopi();
    order.builds.clear();
    for line in &mut order.repo {
        if line.name.as_str() == name {
            line.role = role;
        }
    }

    order
}

fn run() -> BuildRun {
    BuildRun::new("1a2b").unwrap()
}

fn none() -> BTreeSet<PackageName> {
    BTreeSet::new()
}

fn installed(names: &[&str]) -> BTreeSet<PackageName> {
    names.iter().map(|name| package(name)).collect()
}

fn queued(names: &[&str]) -> Vec<PackageName> {
    names.iter().map(|name| package(name)).collect()
}

fn targets(names: &[&str]) -> TargetList<PackageName> {
    TargetList::new(queued(names)).unwrap()
}

/// The plan of `names` queued over `orders`, with no package installed.
fn plan_of(names: &[&str], orders: &[AurOrder]) -> BuildPlan {
    plan(&queued(names), orders, &run(), &none()).unwrap()
}

/// The repository lines of the order that are dependencies or MAKE.
fn dependencies(order: &AurOrder) -> TargetList<PackageName> {
    TargetList::new(
        order
            .repo
            .iter()
            .filter(|line| line.role != Role::Target)
            .map(|line| line.name.clone()),
    )
    .unwrap()
}

/// The MAKE lines of the order, sorted as the plan keeps them.
fn make_names(order: &AurOrder) -> Vec<PackageName> {
    let mut names: Vec<PackageName> = order
        .repo
        .iter()
        .filter(|line| line.role == Role::Make)
        .map(|line| line.name.clone())
        .collect();

    names.sort();
    names
}

fn root(action: RootAction) -> PlannedAction {
    PlannedAction::Root(action)
}

fn user(action: UserAction) -> PlannedAction {
    PlannedAction::User(action)
}

fn build_aur(base: &str) -> PlannedAction {
    user(UserAction::BuildAur {
        base: package(base),
        run: run(),
    })
}

fn install_built(names: &[&str], reason: InstallReason) -> PlannedAction {
    root(RootAction::InstallBuilt {
        run: run(),
        packages: targets(names),
        reason,
    })
}

fn mark_dependencies(names: &[&str]) -> PlannedAction {
    root(RootAction::MarkDependencies {
        packages: targets(names),
    })
}

fn review(bases: &[&str]) -> PlannedAction {
    user(UserAction::ReviewAur {
        bases: targets(bases),
    })
}

fn remove_make_dependencies(names: Vec<PackageName>) -> PlannedAction {
    root(RootAction::RemoveMakeDependencies {
        packages: TargetList::new(names).unwrap(),
    })
}

fn is_review(action: &PlannedAction) -> bool {
    matches!(
        action,
        PlannedAction::User(UserAction::ReviewAur { bases: _ })
    )
}

fn is_removal(action: &PlannedAction) -> bool {
    matches!(
        action,
        PlannedAction::Root(RootAction::RemoveMakeDependencies { packages: _ })
    )
}

fn install_dependencies(packages: TargetList<PackageName>) -> PlannedAction {
    root(RootAction::InstallDependencies { packages })
}

/// The names an action installs or builds.
fn names_of(action: &PlannedAction) -> Vec<PackageName> {
    let mut names = Vec::new();

    if let PlannedAction::Root(
        RootAction::InstallPackages { packages }
        | RootAction::InstallDependencies { packages }
        | RootAction::RemoveMakeDependencies { packages }
        | RootAction::InstallBuilt {
            run: _,
            packages,
            reason: _,
        },
    ) = action
    {
        names.extend(packages.iter().cloned());
    }

    if let PlannedAction::User(UserAction::BuildAur { base, run: _ }) = action {
        names.push(base.clone());
    }

    if let PlannedAction::User(UserAction::ReviewAur { bases }) = action {
        names.extend(bases.iter().cloned());
    }

    names
}

/// Every name an order mentions, as a repository line or a build.
fn named_by(orders: &[AurOrder]) -> BTreeSet<PackageName> {
    orders
        .iter()
        .flat_map(|order| {
            order.repo.iter().map(|line| line.name.clone()).chain(
                order
                    .builds
                    .iter()
                    .flat_map(|build| [build.base.clone(), build.name.clone()]),
            )
        })
        .collect()
}
