use super::listings::{
    BARE_METAL, INSTALLED, LIVE, LVM_ON_LUKS, laptop_from_the_live_iso, lvm_with_only_swap,
};
use crate::features::bootrepair::{Encryption, Inventory, parse_lsblk};

fn systems(inventory: &Inventory) -> Vec<(&str, &str, Encryption, bool)> {
    inventory
        .systems
        .iter()
        .map(|system| {
            (
                system.device(),
                system.file_system.as_str(),
                system.encryption,
                system.in_use,
            )
        })
        .collect()
}

#[test]
fn the_live_iso_skips_its_own_medium_and_lists_the_empty_disk() {
    let inventory = parse_lsblk(LIVE).unwrap();

    assert!(inventory.systems.is_empty(), "{:?}", inventory.systems);
    assert!(inventory.esps.is_empty(), "{:?}", inventory.esps);

    let disks: Vec<_> = inventory
        .disks
        .iter()
        .map(|disk| (disk.device.as_str(), disk.size, disk.in_use))
        .collect();

    assert_eq!(disks, [("/dev/vda", 32_212_254_720, false)]);
}

#[test]
fn empty_card_readers_and_zram_are_no_disks_and_the_running_disk_is_busy() {
    let inventory = parse_lsblk(BARE_METAL).unwrap();

    let disks: Vec<_> = inventory
        .disks
        .iter()
        .map(|disk| (disk.device.as_str(), disk.model.as_str(), disk.in_use))
        .collect();

    assert_eq!(
        disks,
        [
            ("/dev/nvme0n1", "WD_BLACK SN8100 2000GB", true),
            ("/dev/nvme1n1", "MTFDKBA1T0QGN-1BN1AABGA", false),
        ]
    );

    assert_eq!(
        systems(&inventory),
        [
            ("/dev/mapper/garuda", "btrfs", Encryption::Unlocked, true),
            (
                "/dev/mapper/sn8100_nix",
                "btrfs",
                Encryption::Unlocked,
                true
            ),
            ("/dev/nvme1n1p2", "btrfs", Encryption::None, false),
        ]
    );

    let esps: Vec<_> = inventory
        .esps
        .iter()
        .map(|esp| (esp.device.as_str(), esp.disk.as_str(), esp.in_use))
        .collect();

    assert_eq!(
        esps,
        [
            ("/dev/nvme0n1p2", "/dev/nvme0n1", true),
            ("/dev/nvme1n1p1", "/dev/nvme1n1", false),
        ]
    );

    let other = inventory.system("/dev/nvme1n1p2").unwrap();

    assert!(other.asks_layout());
    assert_eq!(
        inventory.usual_esp(other).unwrap().device.as_str(),
        "/dev/nvme1n1p1"
    );
    assert_eq!(
        inventory.usual_disk(other).unwrap().device.as_str(),
        "/dev/nvme1n1"
    );
}

#[test]
fn luks_partitions_without_their_mapping_are_locked_systems() {
    let inventory = parse_lsblk(&laptop_from_the_live_iso()).unwrap();

    assert_eq!(
        systems(&inventory),
        [
            ("/dev/nvme0n1p1", "", Encryption::Locked, false),
            ("/dev/nvme0n1p3", "", Encryption::Locked, false),
            ("/dev/nvme1n1p2", "btrfs", Encryption::None, false),
        ]
    );
    assert!(inventory.system("/dev/nvme0n1p1").unwrap().asks_layout());
}

#[test]
fn volumes_on_a_luks_disk_are_unlocked_systems_and_swap_keeps_no_disk_busy() {
    let mounted = parse_lsblk(LVM_ON_LUKS).unwrap();

    assert!(mounted.system("/dev/mapper/vg0-root").unwrap().in_use);

    let inventory = parse_lsblk(&lvm_with_only_swap()).unwrap();

    assert_eq!(
        systems(&inventory),
        [
            ("/dev/vda2", "btrfs", Encryption::None, true),
            ("/dev/mapper/vg0-root", "ext4", Encryption::Unlocked, false),
            ("/dev/mapper/vg0-home", "ext4", Encryption::Unlocked, false),
        ]
    );

    let root = inventory.system("/dev/mapper/vg0-root").unwrap();

    assert_eq!(root.disks.len(), 1);
    assert_eq!(root.disks.first().unwrap().as_str(), "/dev/vdb");
    assert!(!root.asks_layout());
    assert_eq!(inventory.usual_esp(root), None);
    assert_eq!(
        inventory.usual_disk(root).unwrap().device.as_str(),
        "/dev/vdb"
    );
}

#[test]
fn a_running_btrfs_system_with_subvolumes_is_in_use() {
    let inventory = parse_lsblk(INSTALLED).unwrap();

    assert_eq!(
        systems(&inventory),
        [("/dev/vda2", "btrfs", Encryption::None, true)]
    );
    assert!(inventory.esps.first().unwrap().in_use);
    assert!(inventory.system("/dev/vda2").unwrap().asks_layout());

    let disks: Vec<_> = inventory
        .disks
        .iter()
        .map(|disk| disk.device.as_str())
        .collect();

    assert_eq!(disks, ["/dev/vda"]);
}

#[test]
fn text_that_is_no_listing_is_an_error_and_odd_devices_are_skipped() {
    parse_lsblk("lsblk: not found").unwrap_err();
    parse_lsblk("{}").unwrap_err();

    let odd = r#"{"blockdevices":[
        {"path":"/dev/sda","type":"disk","size":1073741824,"children":[
            {"path":"/dev/sda1; rm -rf /","type":"part","fstype":"ext4"},
            {"path":"/dev/disk/by-label/x","type":"part","fstype":"ext4"},
            {"path":null,"type":"part","fstype":"ext4"},
            {"path":"/dev/mapper/a b","type":"lvm","fstype":"ext4"},
            {"path":"/dev/sda5","type":"part","fstype":"ntfs"}
        ]},
        {"path":"/dev/md0","type":"raid1","size":1073741824,"fstype":"ext4"},
        {"path":"/etc/passwd","type":"disk","size":1073741824}
    ]}"#;

    let inventory = parse_lsblk(odd).unwrap();

    assert!(inventory.systems.is_empty(), "{:?}", inventory.systems);
    assert_eq!(inventory.disks.len(), 1);
}
