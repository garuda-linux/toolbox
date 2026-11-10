//! `drop_done_one_offs` plans each one-off change alone and drops it when
//! all those actions succeeded in the batch. That holds only while the
//! planner keeps a one-off change's actions as they are inside a batch,
//! never merged with another change's.

use toolbox_plan::{DkmsBuild, DkmsModuleName, DkmsVersion, KernelRelease};

use crate::domain::{Domain as _, Queued, Toggle};
use crate::domains::{AccountLocks, DkmsBuilds, Packages};
use crate::features::maintenance::{Operation, Operations, Task};
use crate::test_support::{all_steps_succeeded, package, user_name};
use crate::{ChangeSet, Snapshot};

fn build(module: &str) -> DkmsBuild {
    DkmsBuild {
        module: DkmsModuleName::new(module).unwrap(),
        version: DkmsVersion::new("1.0").unwrap(),
        kernel: KernelRelease::new("6.12.9-1-lts").unwrap(),
    }
}

/// Two changes of every one-off domain in one batch with other work.
fn batch() -> ChangeSet {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<DkmsBuilds>(build("vboxhost"), Queued, &empty);
    changes.want::<DkmsBuilds>(build("acpi_call"), Queued, &empty);
    changes.want::<AccountLocks>(user_name("nina"), Toggle::On, &empty);
    changes.want::<AccountLocks>(user_name("ada"), Toggle::Off, &empty);
    Operations::want(&mut changes, Task::RemoveOrphans, &empty);
    Operations::want(&mut changes, Task::CleanPackageCache, &empty);
    changes.want::<Packages>(package("linux-lts-headers"), Toggle::On, &empty);
    changes
}

#[test]
fn every_one_off_change_leaves_the_queue_once_the_whole_batch_succeeded() {
    let mut changes = batch();

    all_steps_succeeded(&mut changes);

    assert_eq!(changes.queued::<DkmsBuilds>(), []);
    assert_eq!(changes.queued::<AccountLocks>(), []);
    assert!(
        changes
            .wish::<Operations>(&Operation::RemoveOrphans)
            .is_none()
    );
    assert!(
        changes
            .wish::<Operations>(&Operation::CleanPackageCache)
            .is_none()
    );
    assert_eq!(
        changes.queued::<Packages>().len(),
        1,
        "a state change waits for the next probe"
    );
}

/// A new one-off domain needs its two changes in [`batch`] too.
#[test]
fn the_batch_holds_every_one_off_domain() {
    let mut covered = vec![AccountLocks::ID, DkmsBuilds::ID, Operations::ID];
    covered.sort_unstable();

    assert_eq!(crate::domains::all().unwrap().one_offs(), covered);
}
