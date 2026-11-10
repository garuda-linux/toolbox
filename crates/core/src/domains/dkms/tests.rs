use toolbox_plan::{DkmsBuild, DkmsModuleName, DkmsVersion, KernelRelease};

use super::DkmsBuilds;
use crate::domain::{Queued, Toggle};
use crate::domains::Packages;
use crate::test_support::{commands, package, restored};
use crate::{ChangeSet, Snapshot, plan};

fn build(module: &str, version: &str, kernel: &str) -> DkmsBuild {
    DkmsBuild {
        module: DkmsModuleName::new(module).unwrap(),
        version: DkmsVersion::new(version).unwrap(),
        kernel: KernelRelease::new(kernel).unwrap(),
    }
}

#[test]
fn builds_run_after_the_packages_that_bring_the_headers() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<DkmsBuilds>(
        build("vboxhost", "7.1.6_OSE", "6.12.9-1-lts"),
        Queued,
        &empty,
    );
    changes.want::<DkmsBuilds>(build("acpi_call", "1.2.2", "6.12.9-1-lts"), Queued, &empty);
    changes.want::<Packages>(package("linux-lts-headers"), Toggle::On, &empty);

    assert_eq!(
        commands(&plan(&changes)),
        [
            "pacman -S --needed --noconfirm -- linux-lts-headers",
            "dkms install acpi_call/1.2.2 -k 6.12.9-1-lts",
            "dkms install vboxhost/7.1.6_OSE -k 6.12.9-1-lts",
        ]
    );
}

#[test]
fn a_queued_build_can_be_taken_back() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<DkmsBuilds>(
        build("nvidia", "550.90.07", "6.14.0-zen1-1-zen"),
        Queued,
        &empty,
    );
    assert_eq!(changes.len(), 1);
    assert_eq!(changes.views()[0].key, "nvidia/550.90.07 6.14.0-zen1-1-zen");

    changes.remove::<DkmsBuilds>(&build("nvidia", "550.90.07", "6.14.0-zen1-1-zen"));
    assert!(changes.is_empty());
}

/// No probe in the snapshot sees a build, so only a successful apply ends it.
#[test]
fn builds_stay_through_snapshots_and_leave_after_a_successful_apply() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<DkmsBuilds>(build("zfs", "2.3.1", "6.14.0-arch1-1"), Queued, &empty);

    changes.reconcile(&empty);
    assert_eq!(changes.len(), 1);

    crate::test_support::all_steps_succeeded(&mut changes);
    assert!(changes.is_empty());
}

#[test]
fn builds_survive_a_save_and_restore() {
    let empty = Snapshot::default();
    let mut changes = ChangeSet::default();
    changes.want::<DkmsBuilds>(build("zfs", "2.3.1", "6.14.0-arch1-1"), Queued, &empty);

    let (restored, problems) = restored(&changes, &empty);

    assert_eq!(problems, []);
    assert_eq!(restored.views(), changes.views());
}
