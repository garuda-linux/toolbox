//! The packages `toolbox_plan::PROTECTED` keeps: no page and no command
//! removes them, neither by name nor as a dependency `pacman -Rns` takes
//! along. Every removal of a package asks here first.

#[cfg(test)]
mod tests;

use toolbox_plan::{PackageName, Protection, protection};
use toolbox_state::packages::PackageGraph;

use super::graph::takes_along;
use crate::Snapshot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CascadeRefusal {
    /// The dependency graph is not read, so what goes along is unknown.
    Unread,
    /// The removal takes this protected dependency along.
    TakesAlong(Protection, PackageName),
}

/// # Errors
///
/// The reason `package` stays installed.
pub fn may_remove(package: &PackageName) -> Result<(), Protection> {
    protection(package).map_or(Ok(()), Err)
}

/// Checks what one `pacman -Rns` over all of `targets` takes along, so
/// pass every removal of the batch together.
///
/// # Errors
///
/// [`CascadeRefusal`] without the graph or for the first protected
/// dependency that would go.
pub fn may_remove_with_cascade(
    targets: &[PackageName],
    snapshot: &Snapshot,
) -> Result<(), CascadeRefusal> {
    let graph = snapshot
        .part::<PackageGraph>()
        .ok_or(CascadeRefusal::Unread)?;

    takes_along(targets, &graph.0)
        .into_iter()
        .find_map(|along| protection(&along).map(|why| CascadeRefusal::TakesAlong(why, along)))
        .map_or(Ok(()), Err)
}
