//! The guard of every queued package removal, however it came.
//!
//! A click, a command, the session bus or a saved queue. The Kernels
//! guard keeps the kernels, this one the protected list and what
//! `pacman -Rns` takes along.

#[cfg(test)]
mod tests;

use toolbox_plan::PackageName;
use toolbox_state::packages::PackageGraph;

use super::queue::{self, Refusal};
use crate::domains::Packages;
use crate::features::Refused;
use crate::{ChangeSet, Snapshot};

/// Drops the removals the package rules refuse. Returns how many went,
/// `None` while a removal waits for the graph.
///
/// The removals are checked in queue order against the batch so far, so
/// of two that free a protected dependency only together the later goes.
pub fn recheck(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    let removals = queue::queued_removals(changes);

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
    let mut batch = kept.to_vec();
    batch.push(package.clone());

    queue::protection_over(std::slice::from_ref(package), &batch, snapshot)
        .err()
        .map(|refused| why(&refused))
}

fn why(refused: &Refused<Refusal, PackageName>) -> String {
    match refused.refusal {
        Refusal::Protected(why) => format!("it is protected ({why:?})"),
        Refusal::TakesAlong(why) => {
            format!("it takes the protected {} along ({why:?})", refused.subject)
        }
        Refusal::Unread => "the dependencies are not known".to_owned(),
        other @ (Refusal::NotOffered
        | Refusal::RunningKernel
        | Refusal::RunningUnknown
        | Refusal::LastKernel) => format!("{other:?}"),
    }
}
