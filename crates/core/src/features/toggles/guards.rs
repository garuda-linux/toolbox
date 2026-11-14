//! The other pages' rules a switch meets: a removal meets the protected
//! list and the Kernels page, a group change the Users page.

use toolbox_catalog::{Check, ToggleEntry};

use super::{Refusal, membership};
use crate::features::packages::protected::{self, CascadeRefusal};
use crate::features::packages::queue::removal_batch;
use crate::features::{Refused, kernels, users};
use crate::{ChangeSet, Snapshot, Toggle};

/// The protected list, what the removal batch takes along and the Kernels
/// page's guards for the packages an entry removes. The `also` packages
/// count as removed even when another entry keeps one, which can refuse
/// more than pacman would, never less.
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

    let removed: Vec<_> = std::iter::once(name).chain(entry.also()).cloned().collect();

    for package in &removed {
        protected::may_remove(package)
            .map_err(|why| Refusal::Protected(why).of_package(package))?;
    }

    protected::may_remove_with_cascade(&removal_batch(changes, &removed), snapshot).map_err(
        |refusal| match refusal {
            CascadeRefusal::Unread => Refusal::Unknown.of_package(name),
            CascadeRefusal::TakesAlong(why, along) => Refusal::TakesAlong(why).of_package(&along),
        },
    )?;

    kernels::may_remove(name, changes, snapshot)
        .map_err(|refusal| Refusal::Kernel(refusal).of_package(name))
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
