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
fn a_disk_its_partition_and_a_mapped_device_are_told_apart() {
    let machine = Machine::bare_metal();

    assert_eq!(kind(machine.root(), number(259, 4)).unwrap(), Kind::Disk);
    assert_eq!(
        kind(machine.root(), number(259, 6)).unwrap(),
        Kind::Partition
    );
    assert_eq!(kind(machine.root(), number(253, 0)).unwrap(), Kind::Stacked);
    kind(machine.root(), number(9, 9)).unwrap_err();
}

#[test]
fn a_partition_lies_on_its_disk() {
    let machine = Machine::bare_metal();

    assert_eq!(
        names(&disks(machine.root(), number(259, 4)).unwrap()),
        ["nvme1n1"]
    );
    assert_eq!(
        names(&disks(machine.root(), number(259, 6)).unwrap()),
        ["nvme1n1"]
    );
    assert_eq!(
        names(&disks(machine.root(), number(253, 0)).unwrap()),
        ["nvme0n1"]
    );
}

#[test]
fn lvm_on_luks_leads_back_to_the_disk_below() {
    let machine = Machine::lima();
    let root = machine.number("dm-1");

    assert_eq!(kind(machine.root(), root).unwrap(), Kind::Stacked);
    assert_eq!(names(&disks(machine.root(), root).unwrap()), ["vdb"]);
}

/// Each layer of the live ISO is dm-verity on a data and a hash
/// partition of the same loop device.
#[test]
fn the_layers_of_the_live_system_lie_on_its_loop_device() {
    let machine = Machine::live();

    for layer in ["dm-0", "dm-1", "dm-2", "dm-3"] {
        let number = machine.number(layer);

        assert_eq!(kind(machine.root(), number).unwrap(), Kind::Stacked);
        assert_eq!(names(&disks(machine.root(), number).unwrap()), ["loop0"]);
    }

    assert_eq!(
        kind(machine.root(), machine.number("loop0")).unwrap(),
        Kind::Disk
    );
    assert_eq!(
        kind(machine.root(), machine.number("loop0p1")).unwrap(),
        Kind::Partition
    );
}

#[test]
fn udev_s_record_answers_single_properties() {
    let machine = Machine::bare_metal();

    assert_eq!(
        property(machine.root(), number(259, 5), "ID_PART_ENTRY_TYPE").as_deref(),
        Some(ESP_TYPE)
    );
    assert_eq!(
        property(machine.root(), number(259, 3), "ID_FS_TYPE").as_deref(),
        Some("crypto_LUKS")
    );
    assert_eq!(property(machine.root(), number(259, 5), "ID_FS"), None);
    assert_eq!(property(machine.root(), number(259, 4), "ID_FS_TYPE"), None);
    assert_eq!(property(machine.root(), number(9, 9), "ID_FS_TYPE"), None);
}

#[test]
fn a_captured_mount_table_gives_every_source_and_mount_point() {
    let machine = Machine::lima();
    let table = mounts(machine.root()).unwrap();

    assert_eq!(table.len(), 36);
    assert_eq!(
        table.first(),
        Some(&Mount {
            source: "/dev/vda2".to_owned(),
            point: "/".to_owned(),
        })
    );
    assert_eq!(
        table.last(),
        Some(&Mount {
            source: "/dev/mapper/vg0-root".to_owned(),
            point: "/mnt/fixture".to_owned(),
        })
    );
}

/// Edit: no capture has a blank in a mount point, so the captured
/// `/mnt/fixture` mount is added again at a point with one, written as
/// the kernel escapes it.
#[test]
fn an_escaped_mount_point_is_unescaped() {
    let mut machine = Machine::lima();

    machine.mount_as(
        "/mnt/fixture",
        "/dev/mapper/vg0-root",
        "/run/media/garuda/My Disk",
    );

    assert_eq!(
        mounts(machine.root()).unwrap().last(),
        Some(&Mount {
            source: "/dev/mapper/vg0-root".to_owned(),
            point: "/run/media/garuda/My Disk".to_owned(),
        })
    );
}

/// Edit: a line without the ` - ` separator appended to the captured
/// table of the Garuda install.
#[test]
fn a_broken_line_in_the_mount_table_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let captured = include_str!("../fixtures/installed/mountinfo");

    std::fs::create_dir_all(dir.path().join("proc/self")).unwrap();
    std::fs::write(
        dir.path().join("proc/self/mountinfo"),
        format!("{captured}broken line\n"),
    )
    .unwrap();

    let table = mounts(dir.path()).unwrap();

    assert_eq!(table.len(), captured.lines().count());
    assert_eq!(table.first().unwrap().source, "/dev/vda2");
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

/// Edit: ten device-mapper devices stacked on the captured LVM volume,
/// deeper than any real system stacks them.
#[test]
fn a_stack_deeper_than_any_real_one_is_refused() {
    let mut machine = Machine::lima();
    let mut below = "dm-1".to_owned();

    for level in 10..20_u32 {
        machine.mapped(level, &below);
        below = format!("dm-{level}");
    }

    kind(machine.root(), number(253, 19)).unwrap();
    disks(machine.root(), number(253, 19)).unwrap_err();
}
