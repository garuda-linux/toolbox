//! An estimate for the question before a removal. pacman works out the real set
//! itself and lists it in the Console.

#[cfg(test)]
mod tests;

use std::collections::{BTreeSet, HashMap};

use toolbox_plan::PackageName;
use toolbox_state::packages::{InstallReason, LocalNode};

const VERSION_MARKS: [char; 3] = ['<', '>', '='];

struct Graph<'local> {
    local: &'local [LocalNode],
    by_name: HashMap<&'local str, usize>,
    needs: Vec<BTreeSet<usize>>,
    users: Vec<BTreeSet<usize>>,
}

impl<'local> Graph<'local> {
    fn new(local: &'local [LocalNode]) -> Self {
        let by_name: HashMap<&str, usize> = local
            .iter()
            .enumerate()
            .map(|(index, node)| (node.name.as_str(), index))
            .collect();

        let mut providers: HashMap<&str, Vec<usize>> = HashMap::new();
        for (index, node) in local.iter().enumerate() {
            for provided in &node.provides {
                providers
                    .entry(dependency_name(provided))
                    .or_default()
                    .push(index);
            }
        }

        // A package of the very name wins over the ones that provide it,
        // as libalpm looks it up first.
        let resolve = |dependency: &str| -> Vec<usize> {
            let wanted = dependency_name(dependency);
            by_name.get(wanted).map_or_else(
                || providers.get(wanted).cloned().unwrap_or_default(),
                |&index| vec![index],
            )
        };

        let needs: Vec<BTreeSet<usize>> = local
            .iter()
            .enumerate()
            .map(|(index, node)| {
                node.depends
                    .iter()
                    .flat_map(|dependency| resolve(dependency))
                    .filter(|&needed| needed != index)
                    .collect()
            })
            .collect();

        let mut users = vec![BTreeSet::new(); local.len()];
        for (user, needed) in needs.iter().enumerate() {
            for &package in needed {
                if let Some(of) = users.get_mut(package) {
                    of.insert(user);
                }
            }
        }

        Self {
            local,
            by_name,
            needs,
            users,
        }
    }

    fn users_of(&self, package: usize) -> impl Iterator<Item = &usize> {
        self.users.get(package).into_iter().flatten()
    }

    fn pulled_in(&self, package: usize) -> bool {
        self.local
            .get(package)
            .is_some_and(|node| node.reason == InstallReason::Dependency)
    }

    fn reachable_dependencies(&self, from: &BTreeSet<usize>) -> BTreeSet<usize> {
        let mut found = BTreeSet::new();
        let mut open: Vec<usize> = from.iter().copied().collect();

        while let Some(package) = open.pop() {
            for &needed in self.needs.get(package).into_iter().flatten() {
                if self.pulled_in(needed) && !from.contains(&needed) && found.insert(needed) {
                    open.push(needed);
                }
            }
        }

        found
    }
}

pub fn dependency_name(dependency: &str) -> &str {
    dependency
        .split_once(VERSION_MARKS)
        .map_or(dependency, |(name, _)| name)
}

/// What a removal of `targets` takes along, sorted by name: the
/// dependencies they pulled in that no package outside the removal
/// needs, followed through their own dependencies.
pub fn takes_along(targets: &[PackageName], local: &[LocalNode]) -> Vec<PackageName> {
    let graph = Graph::new(local);
    let removed: BTreeSet<usize> = targets
        .iter()
        .filter_map(|target| graph.by_name.get(target.as_str()).copied())
        .collect();

    let mut along = graph.reachable_dependencies(&removed);

    // Drop what something that stays needs, until nothing changes: a
    // package kept keeps its own dependencies too.
    loop {
        let kept: Vec<usize> = along
            .iter()
            .copied()
            .filter(|&candidate| {
                graph
                    .users_of(candidate)
                    .any(|user| !removed.contains(user) && !along.contains(user))
            })
            .collect();

        if kept.is_empty() {
            break;
        }

        for package in kept {
            along.remove(&package);
        }
    }

    let mut names: Vec<PackageName> = along
        .into_iter()
        .filter_map(|index| local.get(index).map(|node| node.name.clone()))
        .collect();

    names.sort();

    names
}

pub fn orphans(local: &[LocalNode]) -> BTreeSet<PackageName> {
    let graph = Graph::new(local);

    local
        .iter()
        .enumerate()
        .filter(|(index, _)| graph.pulled_in(*index) && graph.users_of(*index).next().is_none())
        .map(|(_, node)| node.name.clone())
        .collect()
}
