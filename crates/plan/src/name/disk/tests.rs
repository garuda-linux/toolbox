use super::MapperRule;
use crate::{BootloaderId, DiskDevice, Name, PartitionDevice, Reason, RootAction, VolumeDevice};

/// What must never reach `mount`, `cryptsetup` or `grub-install`: other
/// paths, options, paths that leave `/dev`, links udev makes, devices of
/// other drivers and anything with a second word or line.
const NEVER_A_DEVICE: [&str; 34] = [
    "",
    "sda",
    "sda1",
    "/",
    "/dev",
    "/dev/",
    "/etc/passwd",
    "/home/nico/disk.img",
    "/dev/../etc/shadow",
    "/dev/sda/../sdb",
    "/dev/./sda",
    "/dev//sda",
    "/dev/sda/",
    "/dev/sda 1",
    "/dev/sda\n/dev/sdb",
    "/dev/sda\0",
    "/dev/SDA",
    "/dev/sd",
    "/dev/sdaaaa",
    "/dev/sda01",
    "/dev/sda0",
    "/dev/sda1234",
    "/dev/nvme0",
    "/dev/nvme0n1p",
    "/dev/nvme0n1p0",
    "/dev/nvme00n1",
    "/dev/mmcblk0boot0",
    "/dev/loop0",
    "/dev/sr0",
    "/dev/zram0",
    "/dev/dm-0",
    "/dev/disk/by-uuid/3f1c9a52-7d0e-4b6a-9c1f-2e8d5a7b4c10",
    "--force",
    "/dev/mapper/",
];

#[test]
fn whole_disks_of_the_known_drivers_pass() {
    for disk in [
        "/dev/sda",
        "/dev/sdz",
        "/dev/sdaa",
        "/dev/vda",
        "/dev/nvme0n1",
        "/dev/nvme12n3",
        "/dev/mmcblk0",
    ] {
        assert_eq!(DiskDevice::new(disk).unwrap().as_str(), disk);
        assert!(PartitionDevice::new(disk).is_err(), "{disk} as a partition");
    }
}

#[test]
fn partitions_of_the_known_drivers_pass() {
    for partition in [
        "/dev/sda1",
        "/dev/sdb12",
        "/dev/sdaa128",
        "/dev/vda2",
        "/dev/nvme0n1p1",
        "/dev/nvme1n2p15",
        "/dev/mmcblk0p2",
    ] {
        assert_eq!(PartitionDevice::new(partition).unwrap().as_str(), partition);
        assert!(DiskDevice::new(partition).is_err(), "{partition} as a disk");
        assert_eq!(VolumeDevice::new(partition).unwrap().as_str(), partition);
    }
}

#[test]
fn nothing_else_is_a_device() {
    for value in NEVER_A_DEVICE {
        assert!(DiskDevice::new(value).is_err(), "disk {value:?}");
        assert!(PartitionDevice::new(value).is_err(), "partition {value:?}");
        assert!(VolumeDevice::new(value).is_err(), "volume {value:?}");
    }
}

#[test]
fn an_empty_device_is_reported_as_empty() {
    assert_eq!(DiskDevice::new("").unwrap_err().reason(), Reason::Empty);
    assert_eq!(
        DiskDevice::new("/etc/passwd").unwrap_err().reason(),
        Reason::UnknownDevice
    );
}

#[test]
fn a_volume_is_a_partition_or_a_mapped_device_and_never_a_disk() {
    for volume in [
        "/dev/mapper/luks-3f1c9a52-7d0e-4b6a-9c1f-2e8d5a7b4c10",
        "/dev/mapper/vg0-root",
        "/dev/mapper/garuda--vg-root",
        "/dev/mapper/cryptroot",
    ] {
        let device = VolumeDevice::new(volume).unwrap();

        assert!(device.is_mapped());
        PartitionDevice::new(volume).unwrap_err();
        DiskDevice::new(volume).unwrap_err();
    }

    VolumeDevice::new("/dev/sda").unwrap_err();
    assert!(!VolumeDevice::new("/dev/sda2").unwrap().is_mapped());
}

#[test]
fn the_repair_mapper_name_obeys_the_rules_of_every_other_one() {
    Name::<MapperRule>::new(crate::repair::REPAIR_MAPPER).unwrap();
}

#[test]
fn a_mapper_name_is_one_plain_path_component() {
    for name in ["cryptroot", "vg0-root", "luks-3f1c9a52", "a.b+c_d"] {
        assert_eq!(Name::<MapperRule>::new(name).unwrap().as_str(), name);
    }

    for name in [
        "",
        ".",
        "..",
        "-q",
        "--key-file=x",
        "a/b",
        "../sda",
        "a b",
        "a\nb",
        "a\0",
        "control",
        "ümlaut",
        "a;b",
        "a$b",
    ] {
        assert!(Name::<MapperRule>::new(name).is_err(), "{name:?}");
        assert!(
            VolumeDevice::new(format!("/dev/mapper/{name}")).is_err(),
            "{name:?} as a volume"
        );
    }

    Name::<MapperRule>::new("a".repeat(127)).unwrap();
    Name::<MapperRule>::new("a".repeat(128)).unwrap_err();
}

#[test]
fn a_partition_is_a_volume() {
    assert_eq!(
        VolumeDevice::from(PartitionDevice::new("/dev/sda2").unwrap()).as_str(),
        "/dev/sda2"
    );
}

#[test]
fn a_boot_loader_name_is_one_folder_name_that_is_no_option() {
    for id in ["Garuda", "GRUB", "arch", "garuda_linux-2"] {
        assert_eq!(BootloaderId::new(id).unwrap().as_str(), id);
    }

    for id in [
        "",
        "-x",
        "--removable",
        ".",
        "..",
        "a/b",
        "Garuda Linux",
        "a.b",
        "a\nb",
        "a=b",
        "ümlaut",
    ] {
        assert!(BootloaderId::new(id).is_err(), "{id:?}");
    }

    BootloaderId::new("a".repeat(32)).unwrap();
    BootloaderId::new("a".repeat(33)).unwrap_err();
}

/// The rules hold on the wire too: a request cannot carry what `new`
/// refuses.
#[test]
fn a_request_with_another_path_as_device_does_not_decode() {
    for device in NEVER_A_DEVICE {
        let line = serde_json::json!({
            "kind": "repair-boot",
            "system": { "root": { "kind": "open", "device": device }, "layout": "plain" },
            "work": { "kind": "regenerate-config" },
        });

        assert!(
            serde_json::from_value::<RootAction>(line).is_err(),
            "{device:?} decoded"
        );
    }
}
