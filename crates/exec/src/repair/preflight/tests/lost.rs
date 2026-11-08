use toolbox_plan::RepairWork;

use super::{PreflightError, open, refusal};
use crate::repair::testing::Machine;

#[test]
fn a_device_the_kernel_lost_is_refused_with_its_name() {
    let mut machine = Machine::live();

    machine.disk("sdz", 8, 99);
    machine.partition("sdz", "sdz1", (8, 100), &[]);
    std::fs::remove_file(machine.root().join("sys/dev/block/8:100")).unwrap();

    let refused = refusal(&machine, open("/dev/sdz1"), RepairWork::RegenerateConfig);

    assert!(
        refused.contains("cannot look up /dev/sdz1 in /sys"),
        "{refused}"
    );
}

#[test]
fn a_mounted_device_the_kernel_lost_is_in_the_way() {
    let mut machine = Machine::live();

    machine.disk("sdz", 8, 99);
    machine.mount("/dev/sdz", "/mnt/usb");
    std::fs::remove_file(machine.root().join("sys/dev/block/8:99")).unwrap();

    let refused = refusal(&machine, open("/dev/sda2"), RepairWork::RegenerateConfig);

    assert!(
        refused.contains("cannot find the disk of /dev/sdz"),
        "{refused}"
    );
}

#[test]
fn failed_lookups_name_the_device_and_the_system_file() {
    let lookup = PreflightError::Lookup {
        path: "/dev/sda2".to_owned(),
        error: std::io::Error::other("gone"),
    };

    assert_eq!(lookup.to_string(), "cannot look up /dev/sda2 in /sys: gone");

    let targets = PreflightError::TargetDisks(std::io::Error::other("gone"));

    assert_eq!(
        targets.to_string(),
        "cannot find the disk of a device in /sys: gone"
    );

    let mounts = PreflightError::MountDisks {
        source: "/dev/sda2".to_owned(),
        error: std::io::Error::other("gone"),
    };

    assert_eq!(
        mounts.to_string(),
        "cannot find the disk of /dev/sda2: gone"
    );
}
