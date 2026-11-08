//! The guards of the domains whose page refuses every change while their
//! part is missing: new groups (the account files), hblock and the Wi-Fi
//! backend. With the part known the page refuses none of them.

#[cfg(test)]
mod tests;

use toolbox_state::probes::{FailedProbes, Probe};

use crate::domains::{Hblock, NewGroups, WifiBackends};
use crate::{ChangeSet, Domain, Snapshot};

pub fn recheck_new_groups(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    drop_without_part::<NewGroups>(changes, snapshot, Probe::Accounts)
}

pub fn recheck_hblock(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    drop_without_part::<Hblock>(changes, snapshot, Probe::Hblock)
}

pub fn recheck_wifi_backend(changes: &mut ChangeSet, snapshot: &Snapshot) -> Option<usize> {
    drop_without_part::<WifiBackends>(changes, snapshot, Probe::WifiBackend)
}

/// Returns how many went, `None` before the first probe: a part that was
/// not read yet is no part the system lacks.
fn drop_without_part<D>(changes: &mut ChangeSet, snapshot: &Snapshot, probe: Probe) -> Option<usize>
where
    D: Domain,
{
    let failed = snapshot.part::<FailedProbes>()?;

    if D::state(snapshot).is_some() {
        return Some(0);
    }

    let why = if failed.contains(probe) {
        "its probe failed"
    } else {
        "the system lacks it"
    };

    let queued = changes.queued::<D>();

    for (item, _wish) in &queued {
        changes.remove::<D>(item);
        tracing::warn!(
            "dropped the restored {} change {} ({}): {why}",
            D::ID,
            D::key(item),
            probe.as_str()
        );
    }

    Some(queued.len())
}
