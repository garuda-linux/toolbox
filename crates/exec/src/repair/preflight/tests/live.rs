//! The Garuda live ISO: no `/run/archiso`, the system's layers are
//! dm-verity devices on the partitions of `loop0`, and the disk it
//! started for is empty. The value objects take no `loop` name, so the
//! disks are checked by number.

use super::super::{Moment, unused};
use crate::repair::testing::Machine;

fn unused_disks(machine: &Machine, names: &[&str]) -> Result<(), String> {
    let targets: Vec<_> = names.iter().map(|name| machine.number(name)).collect();

    unused(&machine.system(), &targets, Moment::Before).map_err(|error| error.to_string())
}

#[test]
fn the_loop_device_the_live_system_runs_from_is_never_a_target() {
    let machine = Machine::live();
    let in_use = Err("/dev/loop0 is in use by the running system (/dev/mapper/garuda-root is mounted at /run/garuda/layers/root), so nothing is written to it".to_owned());

    assert_eq!(unused_disks(&machine, &["loop0"]), in_use);
    assert_eq!(unused_disks(&machine, &["loop0p1"]), in_use);
    assert_eq!(unused_disks(&machine, &["vda", "loop0p9"]), in_use);
}

/// udev names the medium behind `loop0`. sr0 itself is never mounted.
#[test]
fn the_medium_behind_the_live_loop_device_is_in_use() {
    let machine = Machine::live();

    assert_eq!(
        unused_disks(&machine, &["sr0"]),
        Err("/dev/sr0 is in use by the running system (/dev/mapper/garuda-root is mounted at /run/garuda/layers/root), so nothing is written to it".to_owned())
    );
}

/// Edit: `loop0` backed by `vda` instead of `sr0`, as when the ISO was
/// written to a USB stick.
#[test]
fn a_usb_stick_behind_the_live_loop_device_is_in_use() {
    let machine = Machine::live();

    machine.edit_record(
        "loop0",
        "E:ID_LOOP_BACKING_FILENAME=/dev/sr0",
        "E:ID_LOOP_BACKING_FILENAME=/dev/vda",
    );

    assert!(
        unused_disks(&machine, &["vda"])
            .unwrap_err()
            .starts_with("/dev/vda is in use by the running system")
    );
    unused_disks(&machine, &["sr0"]).unwrap();
}

#[test]
fn the_empty_disk_of_the_live_machine_is_unused() {
    let machine = Machine::live();

    unused_disks(&machine, &["vda"]).unwrap();
}

/// The overlay at `/` and the tmpfs below `/run/garuda` have no device,
/// so only the dm-verity layers place the live system on a disk.
#[test]
fn every_layer_of_the_live_system_marks_its_loop_device() {
    let mut machine = Machine::live();

    for layer in ["root", "desktop", "mhwd"] {
        machine.unmount_from(&format!("/dev/mapper/garuda-{layer}"));
    }

    assert!(
        unused_disks(&machine, &["loop0"])
            .unwrap_err()
            .contains("(/dev/mapper/garuda-live is mounted at /run/garuda/layers/live)")
    );

    machine.unmount_from("/dev/mapper/garuda-live");

    unused_disks(&machine, &["loop0"]).unwrap();
    unused_disks(&machine, &["sr0"]).unwrap();
}
