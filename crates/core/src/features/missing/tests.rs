use toolbox_plan::WifiBackend;
use toolbox_state::probes::{FailedProbes, Probe};

use super::{recheck_hblock, recheck_new_groups, recheck_wifi_backend};
use crate::domains::{Accounts, CurrentWifiBackend, Hblock, HblockState, NewGroups, WifiBackends};
use crate::testing::group_name;
use crate::{ChangeSet, Queued, Single, Snapshot, Toggle};

fn restored() -> ChangeSet {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();

    changes.want::<NewGroups>(group_name("crafts"), Queued, &empty);
    changes.want::<Hblock>(Single, Toggle::On, &empty);
    changes.want::<WifiBackends>(Single, WifiBackend::Iwd, &empty);
    changes
}

fn probed(failed: &[Probe]) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(FailedProbes::new(failed.iter().copied()));
    snapshot
}

fn run_all(changes: &mut ChangeSet, snapshot: &Snapshot) -> [Option<usize>; 3] {
    [
        recheck_new_groups(changes, snapshot),
        recheck_hblock(changes, snapshot),
        recheck_wifi_backend(changes, snapshot),
    ]
}

#[test]
fn before_the_first_probe_the_restored_changes_wait() {
    let mut changes = restored();

    assert_eq!(run_all(&mut changes, &Snapshot::default()), [None; 3]);
    assert_eq!(changes.len(), 3);
}

#[test]
fn a_part_whose_probe_failed_drops_its_changes() {
    let mut changes = restored();
    let snapshot = probed(&[Probe::Accounts, Probe::Hblock, Probe::WifiBackend]);

    assert_eq!(run_all(&mut changes, &snapshot), [Some(1); 3]);
    assert!(changes.is_empty());
}

/// The page refuses them on a system without the thing as well, such as
/// one without NetworkManager.
#[test]
fn a_part_the_system_lacks_drops_its_changes_too() {
    let mut changes = restored();

    assert_eq!(run_all(&mut changes, &probed(&[])), [Some(1); 3]);
    assert!(changes.is_empty());
}

#[test]
fn a_part_that_was_read_keeps_its_changes() {
    let mut changes = restored();
    let mut snapshot = probed(&[]);

    snapshot.insert(Accounts::default());
    snapshot.insert(HblockState(false));
    snapshot.insert(CurrentWifiBackend(WifiBackend::WpaSupplicant));

    assert_eq!(run_all(&mut changes, &snapshot), [Some(0); 3]);
    assert_eq!(changes.len(), 3);
}
