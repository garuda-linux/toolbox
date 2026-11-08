mod live;
mod lost;

use std::error::Error as _;

use toolbox_plan::{
    BootloaderId, DiskDevice, InstalledSystem, PackageName, PartitionDevice, RepairRoot,
    RepairWork, RootAction, RootLayout, TargetList, VolumeDevice,
};

use super::{Moment, PreflightError, check, devices};
use crate::repair::testing::{ESP_TYPE, Machine};

const REGENERATE: RepairWork = RepairWork::RegenerateConfig { esp: None };

fn open(device: &str) -> InstalledSystem {
    InstalledSystem {
        root: RepairRoot::Open {
            device: VolumeDevice::new(device).unwrap(),
        },
        layout: RootLayout::Subvolume,
    }
}

fn locked(partition: &str) -> InstalledSystem {
    InstalledSystem {
        root: RepairRoot::Locked {
            partition: PartitionDevice::new(partition).unwrap(),
        },
        layout: RootLayout::Subvolume,
    }
}

fn uefi(esp: &str) -> RepairWork {
    RepairWork::InstallUefi {
        esp: PartitionDevice::new(esp).unwrap(),
        id: BootloaderId::new("Garuda").unwrap(),
    }
}

fn bios(disk: &str) -> RepairWork {
    RepairWork::InstallBios {
        disk: DiskDevice::new(disk).unwrap(),
    }
}

fn repair(system: InstalledSystem, work: RepairWork) -> RootAction {
    RootAction::RepairBoot {
        system,
        work,
        passphrase: None,
    }
}

fn refusal(machine: &Machine, system: InstalledSystem, work: RepairWork) -> String {
    check(&machine.system(), &repair(system, work))
        .unwrap_err()
        .to_string()
}

/// Edit: every mount of the laptop's own disks dropped, as when it runs
/// a live system and nothing of its disks is mounted.
fn laptop_from_live() -> Machine {
    let mut machine = Machine::bare_metal();

    machine.unmount_from("/dev/");
    machine
}

/// `nvme1n1` holds an EFI partition and btrfs that the laptop does not
/// mount.
#[test]
fn repairs_of_an_unused_disk_pass() {
    let machine = Machine::bare_metal();

    for (system, work) in [
        (open("/dev/nvme1n1p2"), REGENERATE),
        (open("/dev/nvme1n1p2"), uefi("/dev/nvme1n1p1")),
        (open("/dev/nvme1n1p2"), bios("/dev/nvme1n1")),
    ] {
        check(&machine.system(), &repair(system, work)).unwrap();
    }
}

#[test]
fn a_locked_or_mapped_root_of_an_unused_disk_passes() {
    let machine = laptop_from_live();

    for (system, work) in [
        (locked("/dev/nvme0n1p3"), uefi("/dev/nvme0n1p2")),
        (open("/dev/mapper/sn8100_nix"), uefi("/dev/nvme0n1p2")),
        (open("/dev/mapper/garuda"), bios("/dev/nvme0n1")),
    ] {
        check(&machine.system(), &repair(system, work)).unwrap();
    }
}

#[test]
fn other_actions_are_not_looked_at() {
    let machine = Machine::new();
    let action = RootAction::InstallPackages {
        packages: TargetList::new([PackageName::new("grub").unwrap()]).unwrap(),
    };

    check(&machine.system(), &action).unwrap();
}

#[test]
fn a_name_without_a_block_device_behind_it_is_refused() {
    let machine = Machine::bare_metal();

    assert_eq!(
        refusal(&machine, open("/dev/nvme2n1p2"), REGENERATE),
        "/dev/nvme2n1p2 is no block device"
    );
    assert_eq!(
        refusal(&machine, open("/dev/nvme1n1p2"), bios("/dev/vda")),
        "/dev/vda is no block device"
    );
    assert_eq!(
        refusal(&machine, open("/dev/mapper/absent"), REGENERATE),
        "/dev/mapper/absent is no block device"
    );
}

/// A name of the right shape is not enough: the kernel decides what the
/// device is. Edit: `/sys` of `nvme1n1p2` without its `partition` file
/// and `/sys` of `nvme1n1` with one, so each name lies about its kind.
#[test]
fn a_device_of_another_kind_than_its_role_is_refused() {
    let machine = Machine::bare_metal();
    let block = machine.root().join("sys/dev/block");

    std::fs::remove_file(block.join("259:6/partition")).unwrap();
    std::fs::write(block.join("259:4/partition"), "1\n").unwrap();

    assert_eq!(
        refusal(&machine, open("/dev/nvme1n1p2"), REGENERATE),
        "/dev/nvme1n1p2 is no partition"
    );
    assert_eq!(
        refusal(&machine, open("/dev/nvme0n1p2"), bios("/dev/nvme1n1")),
        "/dev/nvme1n1 is no whole disk"
    );
}

#[test]
fn only_an_efi_system_partition_takes_the_boot_loader() {
    let machine = Machine::bare_metal();

    assert_eq!(
        refusal(&machine, open("/dev/nvme1n1p2"), uefi("/dev/nvme1n1p2")),
        "/dev/nvme1n1p2 is no EFI system partition"
    );

    // Edit: the EFI partition's record says ext4 instead of vfat.
    machine.edit_record("nvme1n1p1", "ID_FS_TYPE=vfat", "ID_FS_TYPE=ext4");

    assert_eq!(
        refusal(&machine, open("/dev/nvme1n1p2"), uefi("/dev/nvme1n1p1")),
        "/dev/nvme1n1p1 is no EFI system partition"
    );
}

/// Edit: the GPT type of the EFI partition replaced with the type udev
/// writes for an MBR one.
#[test]
fn an_esp_of_an_mbr_disk_counts() {
    let machine = Machine::bare_metal();

    machine.edit_record(
        "nvme1n1p1",
        &format!("ID_PART_ENTRY_TYPE={ESP_TYPE}"),
        "ID_PART_ENTRY_TYPE=0xef",
    );

    check(
        &machine.system(),
        &repair(open("/dev/nvme1n1p2"), uefi("/dev/nvme1n1p1")),
    )
    .unwrap();
}

/// Edit: `/sys/firmware/efi` removed, as on a system started through
/// BIOS.
#[test]
fn a_uefi_install_needs_a_live_system_started_through_uefi() {
    let machine = Machine::bare_metal();

    std::fs::remove_dir_all(machine.root().join("sys/firmware/efi")).unwrap();

    assert!(
        refusal(&machine, open("/dev/nvme1n1p2"), uefi("/dev/nvme1n1p1"))
            .starts_with("the live system was not started through UEFI")
    );
    check(
        &machine.system(),
        &repair(open("/dev/nvme1n1p2"), bios("/dev/nvme1n1")),
    )
    .unwrap();
}

#[test]
fn only_a_luks_partition_is_unlocked() {
    let machine = Machine::bare_metal();

    assert_eq!(
        refusal(&machine, locked("/dev/nvme1n1p2"), REGENERATE),
        "/dev/nvme1n1p2 is no LUKS partition"
    );
}

/// The test mode on an installed system: its own disk is mounted.
#[test]
fn a_system_cannot_repair_the_disk_it_runs_from() {
    let machine = Machine::lima();

    assert_eq!(
        refusal(&machine, open("/dev/vda2"), REGENERATE),
        "/dev/vda is in use by the running system (/dev/vda2 is mounted at /), so nothing is written to it"
    );
}

#[test]
fn a_mapped_root_is_traced_to_the_disks_below_it() {
    let machine = Machine::bare_metal();

    assert!(
        refusal(&machine, open("/dev/mapper/garuda"), REGENERATE)
            .starts_with("/dev/nvme0n1 is in use")
    );
}

#[test]
fn a_mount_of_a_mapped_device_marks_the_disks_below_it() {
    let machine = Machine::bare_metal();

    assert_eq!(
        refusal(&machine, open("/dev/nvme1n1p2"), uefi("/dev/nvme0n1p2")),
        "/dev/nvme0n1 is in use by the running system (/dev/mapper/sn8100_nix is mounted at /nix), so nothing is written to it"
    );
}

/// Edit: the captured `/dev` mount added again below the repair's
/// folder.
#[test]
fn a_mount_left_by_an_earlier_repair_stops_the_next_one() {
    let mut machine = Machine::bare_metal();

    machine.mount_as(
        "/dev",
        "devtmpfs",
        "/run/garuda-toolbox/boot-repair/root/dev",
    );

    assert_eq!(
        refusal(&machine, open("/dev/nvme1n1p2"), REGENERATE),
        "/run/garuda-toolbox/boot-repair/root/dev is still mounted from an earlier repair, unmount it or restart the live system"
    );
}

/// Edit: the captured `/boot` mount added again from a partition the
/// machine has no node for.
#[test]
fn a_mount_without_a_device_node_stops_the_repair() {
    let mut machine = Machine::bare_metal();

    machine.mount_as("/boot", "/dev/sdc9", "/mnt");

    assert_eq!(
        refusal(&machine, open("/dev/nvme1n1p2"), REGENERATE),
        "/dev/sdc9 is mounted at /mnt but has no device node, so no disk is known to be unused"
    );
}

/// Edit: the unused disk mounted below the repair's folder as the repair
/// mounts it, written like the captured `/data`, `/dev` and `/boot`
/// mounts.
fn mid_repair() -> Machine {
    let mut machine = Machine::bare_metal();
    let root = "/run/garuda-toolbox/boot-repair/root";

    machine.mount_as("/data", "/dev/nvme1n1p2", root);
    machine.mount_as("/dev", "devtmpfs", &format!("{root}/dev"));
    machine.mount_as("/boot", "/dev/nvme1n1p1", &format!("{root}/boot/efi"));
    machine
}

fn second_look(machine: &Machine, work: &RepairWork) -> Result<(), String> {
    devices(
        &machine.system(),
        &open("/dev/nvme1n1p2"),
        work,
        Moment::During,
    )
    .map_err(|error| error.to_string())
}

#[test]
fn the_second_look_passes_over_the_mounts_of_the_repair_itself() {
    let machine = mid_repair();

    for work in [uefi("/dev/nvme1n1p1"), bios("/dev/nvme1n1")] {
        second_look(&machine, &work).unwrap();
    }
}

#[test]
fn the_second_look_refuses_a_name_that_now_belongs_to_a_used_disk() {
    let machine = mid_repair();

    assert!(
        second_look(&machine, &bios("/dev/nvme0n1"))
            .unwrap_err()
            .starts_with("/dev/nvme0n1 is in use")
    );
    assert_eq!(
        second_look(&machine, &bios("/dev/sdc")),
        Err("/dev/sdc is no block device".to_owned())
    );
}

#[test]
fn a_refusal_is_a_typed_error_that_keeps_its_cause() {
    let machine = Machine::bare_metal();
    let missing = check(
        &machine.system(),
        &repair(open("/dev/nvme2n1p2"), REGENERATE),
    )
    .unwrap_err();

    assert!(
        matches!(&missing, PreflightError::NoBlockDevice(path) if path == "/dev/nvme2n1p2"),
        "{missing:?}"
    );
    assert!(missing.source().is_none());

    let unreadable = PreflightError::MountTable(std::io::Error::other("gone"));

    assert_eq!(unreadable.to_string(), "cannot read the mount table: gone");
    assert_eq!(unreadable.source().unwrap().to_string(), "gone");
}
