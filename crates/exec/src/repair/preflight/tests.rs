mod lost;

use std::error::Error as _;

use toolbox_plan::{
    BootloaderId, DiskDevice, InstalledSystem, PackageName, PartitionDevice, RepairRoot,
    RepairWork, RootAction, RootLayout, TargetList, VolumeDevice,
};

use super::{Moment, PreflightError, check, devices};
use crate::repair::testing::{ESP, ESP_TYPE, Machine};

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

#[test]
fn repairs_of_an_unused_disk_pass() {
    let machine = Machine::live();

    for (system, work) in [
        (open("/dev/sda2"), RepairWork::RegenerateConfig),
        (open("/dev/sda2"), uefi("/dev/sda1")),
        (open("/dev/sda2"), bios("/dev/sda")),
        (locked("/dev/sda3"), uefi("/dev/sda1")),
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
    let machine = Machine::live();

    assert_eq!(
        refusal(&machine, open("/dev/sdc2"), RepairWork::RegenerateConfig),
        "/dev/sdc2 is no block device"
    );
    assert_eq!(
        refusal(&machine, open("/dev/sda2"), bios("/dev/vda")),
        "/dev/vda is no block device"
    );
    assert_eq!(
        refusal(
            &machine,
            open("/dev/mapper/absent"),
            RepairWork::RegenerateConfig
        ),
        "/dev/mapper/absent is no block device"
    );
}

/// A name of the right shape is not enough: the kernel decides what the
/// device is.
#[test]
fn a_device_of_another_kind_than_its_role_is_refused() {
    let mut machine = Machine::live();

    // A disk whose name looks like a partition and the other way round.
    machine.disk("sdc1", 8, 33);
    machine.partition("sda", "sdd", (8, 48), &[]);

    assert_eq!(
        refusal(&machine, open("/dev/sdc1"), RepairWork::RegenerateConfig),
        "/dev/sdc1 is no partition"
    );
    assert_eq!(
        refusal(&machine, open("/dev/sda2"), bios("/dev/sdd")),
        "/dev/sdd is no whole disk"
    );
}

#[test]
fn only_an_efi_system_partition_takes_the_boot_loader() {
    let mut machine = Machine::live();

    machine.partition(
        "sda",
        "sda4",
        (8, 4),
        &[("ID_FS_TYPE", "ext4"), ("ID_PART_ENTRY_TYPE", ESP_TYPE)],
    );

    assert_eq!(
        refusal(&machine, open("/dev/sda2"), uefi("/dev/sda2")),
        "/dev/sda2 is no EFI system partition"
    );
    assert_eq!(
        refusal(&machine, open("/dev/sda2"), uefi("/dev/sda4")),
        "/dev/sda4 is no EFI system partition"
    );
}

#[test]
fn an_esp_of_an_mbr_disk_counts() {
    let mut machine = Machine::live();

    machine.partition(
        "sda",
        "sda5",
        (8, 5),
        &[("ID_FS_TYPE", "vfat"), ("ID_PART_ENTRY_TYPE", "0xef")],
    );

    check(
        &machine.system(),
        &repair(open("/dev/sda2"), uefi("/dev/sda5")),
    )
    .unwrap();
}

#[test]
fn a_uefi_install_needs_a_live_system_started_through_uefi() {
    let mut machine = Machine::new();

    machine.disk("sda", 8, 0);
    machine.partition("sda", "sda1", (8, 1), &ESP);
    machine.partition("sda", "sda2", (8, 2), &[]);

    assert!(
        refusal(&machine, open("/dev/sda2"), uefi("/dev/sda1"))
            .starts_with("the live system was not started through UEFI")
    );
    check(
        &machine.system(),
        &repair(open("/dev/sda2"), bios("/dev/sda")),
    )
    .unwrap();
}

#[test]
fn only_a_luks_partition_is_unlocked() {
    let machine = Machine::live();

    assert_eq!(
        refusal(&machine, locked("/dev/sda2"), RepairWork::RegenerateConfig),
        "/dev/sda2 is no LUKS partition"
    );
}

#[test]
fn the_disk_the_live_system_runs_from_is_never_a_target() {
    let mut machine = Machine::live();

    machine.partition("sdb", "sdb2", (8, 18), &ESP);
    machine.partition("sdb", "sdb3", (8, 19), &[("ID_FS_TYPE", "ext4")]);

    let in_use = "/dev/sdb is in use by the running system (/dev/sdb1 is mounted at /run/archiso/bootmnt), so nothing is written to it";

    assert_eq!(
        refusal(&machine, open("/dev/sda2"), bios("/dev/sdb")),
        in_use
    );
    assert_eq!(
        refusal(&machine, open("/dev/sda2"), uefi("/dev/sdb2")),
        in_use
    );
    assert_eq!(
        refusal(&machine, open("/dev/sdb3"), RepairWork::RegenerateConfig),
        in_use
    );
}

/// The test mode on an installed system: its own disk is mounted.
#[test]
fn a_system_cannot_repair_the_disk_it_runs_from() {
    let mut machine = Machine::live();

    machine.mount("/dev/sda2", "/");

    assert!(
        refusal(&machine, open("/dev/sda2"), uefi("/dev/sda1"))
            .starts_with("/dev/sda is in use by the running system")
    );
}

#[test]
fn a_mapped_root_is_traced_to_the_disks_below_it() {
    let mut machine = Machine::live();

    machine.mapped("luks-root", 0, &["sda3"]);
    machine.mapped("live-rw", 1, &["sdb1"]);

    check(
        &machine.system(),
        &repair(open("/dev/mapper/luks-root"), uefi("/dev/sda1")),
    )
    .unwrap();
    assert!(
        refusal(
            &machine,
            open("/dev/mapper/live-rw"),
            RepairWork::RegenerateConfig
        )
        .starts_with("/dev/sdb is in use")
    );
}

#[test]
fn a_mount_of_a_mapped_device_marks_the_disks_below_it() {
    let mut machine = Machine::live();

    machine.mapped("data", 0, &["sda3"]);
    machine.mount("/dev/mapper/data", "/mnt");

    assert!(
        refusal(&machine, open("/dev/sda2"), RepairWork::RegenerateConfig).starts_with(
            "/dev/sda is in use by the running system (/dev/mapper/data is mounted at /mnt)"
        )
    );
}

#[test]
fn a_mount_left_by_an_earlier_repair_stops_the_next_one() {
    let mut machine = Machine::live();

    machine.mount("devtmpfs", "/run/garuda-toolbox/boot-repair/root/dev");

    assert_eq!(
        refusal(&machine, open("/dev/sda2"), RepairWork::RegenerateConfig),
        "/run/garuda-toolbox/boot-repair/root/dev is still mounted from an earlier repair, unmount it or restart the live system"
    );
}

#[test]
fn a_mount_without_a_device_node_stops_the_repair() {
    let mut machine = Machine::live();

    machine.mount("/dev/sdc9", "/mnt");

    assert_eq!(
        refusal(&machine, open("/dev/sda2"), RepairWork::RegenerateConfig),
        "/dev/sdc9 is mounted at /mnt but has no device node, so no disk is known to be unused"
    );
}

#[test]
fn the_second_look_passes_over_the_mounts_of_the_repair_itself() {
    let mut machine = Machine::live();

    machine.mount("/dev/sda2", "/run/garuda-toolbox/boot-repair/root");
    machine.mount("devtmpfs", "/run/garuda-toolbox/boot-repair/root/dev");
    machine.mount("/dev/sda1", "/run/garuda-toolbox/boot-repair/root/boot/efi");

    for work in [uefi("/dev/sda1"), bios("/dev/sda")] {
        devices(&machine.system(), &open("/dev/sda2"), &work, Moment::During).unwrap();
    }
}

#[test]
fn the_second_look_refuses_a_name_that_now_belongs_to_a_used_disk() {
    let mut machine = Machine::live();

    machine.mount("/dev/sda2", "/run/garuda-toolbox/boot-repair/root");

    assert!(
        devices(
            &machine.system(),
            &open("/dev/sda2"),
            &bios("/dev/sdb"),
            Moment::During
        )
        .unwrap_err()
        .to_string()
        .starts_with("/dev/sdb is in use")
    );
    assert_eq!(
        devices(
            &machine.system(),
            &open("/dev/sda2"),
            &bios("/dev/sdc"),
            Moment::During
        )
        .map_err(|error| error.to_string()),
        Err("/dev/sdc is no block device".to_owned())
    );
}

#[test]
fn a_refusal_is_a_typed_error_that_keeps_its_cause() {
    let machine = Machine::live();
    let missing = check(
        &machine.system(),
        &repair(open("/dev/sdc2"), RepairWork::RegenerateConfig),
    )
    .unwrap_err();

    assert!(
        matches!(&missing, PreflightError::NoBlockDevice(path) if path == "/dev/sdc2"),
        "{missing:?}"
    );
    assert!(missing.source().is_none());

    let unreadable = PreflightError::MountTable(std::io::Error::other("gone"));

    assert_eq!(unreadable.to_string(), "cannot read the mount table: gone");
    assert_eq!(unreadable.source().unwrap().to_string(), "gone");
}
