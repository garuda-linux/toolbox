//! The guard of every queued package removal, however it came.
//!
//! A click, a command, the session bus or a saved queue. The Kernels
//! guard keeps the kernels, this one the protected list and what
//! `pacman -Rns` takes along.

#[cfg(test)]
mod tests;

use toolbox_plan::PackageName;
use toolbox_state::packages::PackageGraph;

use super::protected::{self, CascadeRefusal};
use crate::domains::Packages;
use crate::{ChangeSet, Snapshot, Toggle};

/// Drops the removals the package rules refuse. Returns how many went,
/// `None` while a removal waits for the graph.
///
/// The removals are checked in queue order against the batch so far, so
/// of two that free a protected dependency only together the later goes.
pub fn recheck(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    let removals: Vec<PackageName> = changes
        .queued::<Packages>()
        .into_iter()
        .filter(|(_, wish)| *wish == Toggle::Off)
        .map(|(package, _)| package)
        .collect();

    if removals.is_empty() {
        return Some(0);
    }

    snapshot.part::<PackageGraph>()?;

    let mut kept: Vec<PackageName> = Vec::new();
    let mut dropped = 0;

    for package in removals {
        match refusal(&package, &kept, snapshot) {
            None => kept.push(package),
            Some(why) => {
                changes.remove::<Packages>(&package);
                tracing::warn!("dropped the removal of {package}: {why}");
                dropped += 1;
            }
        }
    }

    Some(dropped)
}

fn refusal(package: &PackageName, kept: &[PackageName], snapshot: &Snapshot) -> Option<String> {
    if let Err(why) = protected::may_remove(package) {
        return Some(format!("it is protected ({why:?})"));
    }

    let mut batch = kept.to_vec();
    batch.push(package.clone());

    match protected::may_remove_with_cascade(&batch, snapshot) {
        Ok(()) => None,
        Err(CascadeRefusal::TakesAlong(why, along)) => {
            Some(format!("it takes the protected {along} along ({why:?})"))
        }
        Err(CascadeRefusal::Unread) => Some("the dependencies are not known".to_owned()),
    }
}
