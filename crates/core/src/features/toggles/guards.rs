//! The other pages' rules a switch meets: a removal meets the package
//! rules, a group change the Users page.

use toolbox_catalog::{Check, ToggleEntry};

use super::{Refusal, membership, removed_package};
use crate::features::packages::queue::{self, Refusal as PackageRefusal};
use crate::features::{Refused, kernels, users};
use crate::{ChangeSet, Snapshot, Toggle};

/// The rules every removal meets, for the package the switch takes away
/// and the entry's `also` packages. Those count as removed even when another
/// entry keeps one, which can refuse more than pacman would, never less.
pub(super) fn removal_allowed(
    entry: &ToggleEntry,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> Result<(), Refused<Refusal>> {
    let Check::Package(name) = entry.check() else {
        return Ok(());
    };

    if wish.is_on() {
        return Ok(());
    }

    let removed: Vec<_> = std::iter::once(removed_package(name, snapshot))
        .chain(entry.also().iter().cloned())
        .collect();

    queue::may_remove_all(&removed, changes, snapshot).map_err(|refused| {
        let refusal = match refused.refusal {
            PackageRefusal::Protected(why) => Refusal::Protected(why),
            PackageRefusal::TakesAlong(why) => Refusal::TakesAlong(why),
            PackageRefusal::RunningKernel => Refusal::Kernel(kernels::Refusal::Running),
            PackageRefusal::RunningUnknown => Refusal::Kernel(kernels::Refusal::RunningUnknown),
            PackageRefusal::LastKernel => Refusal::Kernel(kernels::Refusal::LastKernel),
            PackageRefusal::Unread | PackageRefusal::NotOffered => Refusal::Unknown,
        };

        refusal.of_package(&refused.subject)
    })
}
/// The Users page's guards for the membership a group entry changes.
pub(super) fn account_allowed(
    entry: &ToggleEntry,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> Result<(), Refused<Refusal>> {
    let Check::Group(group) = entry.check() else {
        return Ok(());
    };

    membership(group, snapshot).map_or(Ok(()), |membership| {
        users::membership_allowed(&membership, wish, changes, snapshot)
            .map_err(|refusal| Refusal::of_membership(refusal, &membership))
    })
}
