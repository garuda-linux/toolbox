//! The other pages' rules a switch meets: a removal meets the protected
//! catalog and the Kernels page, a group change the Users page.

use toolbox_catalog::{Check, ToggleEntry};

use super::{Refusal, membership};
use crate::features::{kernels, packages, users};
use crate::{ChangeSet, Snapshot, Toggle};

/// The protected catalog and the Kernels page's guards for the package an
/// entry removes.
pub(super) fn removal_allowed(
    entry: &ToggleEntry,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> Result<(), Refusal> {
    let Check::Package(name) = entry.check() else {
        return Ok(());
    };

    if wish.is_on() {
        return Ok(());
    }

    packages::protected::may_remove(name, snapshot)
        .map_err(|why| Refusal::Protected(why, name.clone()))?;

    kernels::may_remove(name, changes, snapshot)
        .map_err(|refusal| Refusal::Kernel(refusal, name.clone()))
}

/// The Users page's guards for the membership a group entry changes.
pub(super) fn account_allowed(
    entry: &ToggleEntry,
    wish: Toggle,
    snapshot: &Snapshot,
    changes: &ChangeSet,
) -> Result<(), Refusal> {
    let Check::Group(group) = entry.check() else {
        return Ok(());
    };

    membership(group, snapshot).map_or(Ok(()), |membership| {
        users::membership_allowed(&membership, wish, changes, snapshot)
            .map_err(|refusal| Refusal::Account(refusal, membership))
    })
}
