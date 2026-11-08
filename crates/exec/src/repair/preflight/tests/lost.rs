use super::{PreflightError, REGENERATE, open, refusal};
use crate::repair::testing::Machine;

/// Edit: the unused partition's entry removed from `/sys/dev/block`.
#[test]
fn a_device_the_kernel_lost_is_refused_with_its_name() {
    let machine = Machine::bare_metal();

    machine.lose("nvme1n1p2");

    let refused = refusal(&machine, open("/dev/nvme1n1p2"), REGENERATE);

    assert!(
        refused.contains("cannot look up /dev/nvme1n1p2 in /sys"),
        "{refused}"
    );
}

/// Edit: the entry of the partition mounted at `/boot` removed from
/// `/sys/dev/block`.
#[test]
fn a_mounted_device_the_kernel_lost_is_in_the_way() {
    let machine = Machine::bare_metal();

    machine.lose("nvme0n1p2");

    let refused = refusal(&machine, open("/dev/nvme1n1p2"), REGENERATE);

    assert!(
        refused.contains("cannot find the disk of /dev/nvme0n1p2"),
        "{refused}"
    );
}

#[test]
fn failed_lookups_name_the_device_and_the_system_file() {
    let lookup = PreflightError::Lookup {
        path: "/dev/nvme1n1p2".to_owned(),
        error: std::io::Error::other("gone"),
    };

    assert_eq!(
        lookup.to_string(),
        "cannot look up /dev/nvme1n1p2 in /sys: gone"
    );

    let targets = PreflightError::TargetDisks(std::io::Error::other("gone"));

    assert_eq!(
        targets.to_string(),
        "cannot find the disk of a device in /sys: gone"
    );

    let mounts = PreflightError::MountDisks {
        source: "/dev/nvme0n1p2".to_owned(),
        error: std::io::Error::other("gone"),
    };

    assert_eq!(
        mounts.to_string(),
        "cannot find the disk of /dev/nvme0n1p2: gone"
    );
}
