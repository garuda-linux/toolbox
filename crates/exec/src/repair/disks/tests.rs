use std::path::Path;

use super::{DeviceNodes, DeviceNumber, Kind, Mount, Nodes as _, disks, kind, mounts, property};
use crate::repair::testing::{ESP_TYPE, Machine};

const fn number(major: u32, minor: u32) -> DeviceNumber {
    DeviceNumber { major, minor }
}

fn names(found: &std::collections::BTreeSet<String>) -> Vec<&str> {
    found.iter().map(String::as_str).collect()
}

#[test]
fn a_disk_and_its_partition_are_told_apart() {
    let machine = Machine::live();

    assert_eq!(kind(machine.root(), number(8, 0)).unwrap(), Kind::Disk);
    assert_eq!(kind(machine.root(), number(8, 2)).unwrap(), Kind::Partition);
    kind(machine.root(), number(9, 9)).unwrap_err();
}

#[test]
fn a_partition_lies_on_its_disk() {
    let machine = Machine::live();

    assert_eq!(
        names(&disks(machine.root(), number(8, 0)).unwrap()),
        ["sda"]
    );
    assert_eq!(
        names(&disks(machine.root(), number(8, 2)).unwrap()),
        ["sda"]
    );
    assert_eq!(
        names(&disks(machine.root(), number(8, 17)).unwrap()),
        ["sdb"]
    );
}

#[test]
fn lvm_on_luks_leads_back_to_every_disk_below() {
    let mut machine = Machine::live();

    machine.disk("nvme0n1", 259, 0);
    machine.partition("nvme0n1", "nvme0n1p2", (259, 2), &[]);
    machine.mapped("cryptlvm", 0, &["sda3"]);
    machine.mapped("vg0-root", 1, &["dm-0", "nvme0n1p2"]);

    assert_eq!(kind(machine.root(), number(254, 1)).unwrap(), Kind::Stacked);
    assert_eq!(
        names(&disks(machine.root(), number(254, 1)).unwrap()),
        ["nvme0n1", "sda"]
    );
}

#[test]
fn udev_s_record_answers_single_properties() {
    let machine = Machine::live();

    assert_eq!(
        property(machine.root(), number(8, 1), "ID_PART_ENTRY_TYPE").as_deref(),
        Some(ESP_TYPE)
    );
    assert_eq!(
        property(machine.root(), number(8, 3), "ID_FS_TYPE").as_deref(),
        Some("crypto_LUKS")
    );
    assert_eq!(property(machine.root(), number(8, 1), "ID_FS"), None);
    assert_eq!(property(machine.root(), number(8, 0), "ID_FS_TYPE"), None);
}

#[test]
fn the_mount_table_gives_sources_and_unescaped_mount_points() {
    let mut machine = Machine::live();

    machine.mount("/dev/sda2", "/run/media/garuda/My Disk");

    assert_eq!(
        mounts(machine.root()).unwrap(),
        [
            Mount {
                source: "/dev/sdb1".to_owned(),
                point: "/run/archiso/bootmnt".to_owned(),
            },
            Mount {
                source: "overlay".to_owned(),
                point: "/".to_owned(),
            },
            Mount {
                source: "/dev/sda2".to_owned(),
                point: "/run/media/garuda/My Disk".to_owned(),
            },
        ]
    );
}

#[test]
fn a_real_mount_table_line_is_understood() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::create_dir_all(dir.path().join("proc/self")).unwrap();
    std::fs::write(
        dir.path().join("proc/self/mountinfo"),
        "29 1 0:26 /@ / rw,noatime shared:1 - btrfs /dev/nvme0n1p2 rw,ssd,subvol=/@\n\
         31 29 0:5 / /dev rw,nosuid shared:2 - devtmpfs devtmpfs rw,size=4096k\n\
         broken line\n",
    )
    .unwrap();

    let table = mounts(dir.path()).unwrap();

    assert_eq!(table.len(), 2);
    assert_eq!(table.first().unwrap().source, "/dev/nvme0n1p2");
    assert_eq!(table.first().unwrap().point, "/");
}

#[test]
fn a_file_or_a_missing_path_is_no_block_device() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("disk.img");

    std::fs::write(&file, "not a disk").unwrap();

    for follow in [false, true] {
        assert_eq!(DeviceNodes.block_device(&file, follow), None);
        assert_eq!(DeviceNodes.block_device(dir.path(), follow), None);
        assert_eq!(
            DeviceNodes.block_device(Path::new("/dev/mapper/toolbox-test-absent"), follow),
            None
        );
        assert_eq!(
            DeviceNodes.block_device(Path::new("/dev/null"), follow),
            None
        );
    }
}

#[test]
fn a_stack_deeper_than_any_real_one_is_refused() {
    let mut machine = Machine::live();
    let mut below = "sda3".to_owned();

    for level in 10..20_u32 {
        machine.mapped(&format!("level{level}"), level, &[&below]);
        below = format!("dm-{level}");
    }

    kind(machine.root(), number(254, 19)).unwrap();
    disks(machine.root(), number(254, 19)).unwrap_err();
}
