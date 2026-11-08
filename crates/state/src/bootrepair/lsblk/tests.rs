use super::{LSBLK_ARGUMENTS, parse_lsblk};
use crate::bootrepair::Encryption;

/// `lsblk` with [`LSBLK_ARGUMENTS`] on the Garuda live ISO in QEMU with
/// an empty 30 GiB disk.
const LIVE: &str = include_str!("../fixtures/live.json");
/// The same on the installed Lima VM, one btrfs system on `/dev/vda`.
const LIMA_BTRFS: &str =
    include_str!("../../../../core/src/features/bootrepair/fixtures/lima-btrfs.json");

/// The same with a second disk, LUKS on the whole disk and LVM inside.
const LIMA_LVM_ON_LUKS: &str =
    include_str!("../../../../core/src/features/bootrepair/fixtures/lima-lvm-on-luks.json");

const LIVE_DISK_SIZE: u64 = 32_212_254_720;
const LIMA_DISK_SIZE: u64 = 21_474_836_480;
const LIMA_ROOT_SIZE: u64 = 20_935_868_416;

#[test]
fn lsblk_is_asked_for_sizes_in_bytes() {
    assert!(LSBLK_ARGUMENTS.contains(&"--bytes"), "{LSBLK_ARGUMENTS:?}");
}

#[test]
fn a_size_in_bytes_is_kept_as_a_number_for_the_page_to_word() {
    let inventory = parse_lsblk(LIMA_BTRFS).unwrap();

    assert_eq!(inventory.disks[0].size, LIMA_DISK_SIZE);
    assert_eq!(inventory.systems[0].size, LIMA_ROOT_SIZE);
}

/// Edit: the size of `/dev/vda2` in quotes.
#[test]
fn an_lsblk_that_quotes_its_bytes_is_read_too() {
    let quoted = LIMA_BTRFS.replace(
        &format!("\"size\": {LIMA_ROOT_SIZE},"),
        &format!("\"size\": \"{LIMA_ROOT_SIZE}\","),
    );

    assert_ne!(quoted, LIMA_BTRFS);
    let inventory = parse_lsblk(&quoted).unwrap();

    assert_eq!(inventory.systems[0].size, LIMA_ROOT_SIZE);
}

/// Edit: the size of `/dev/vda2` as `"19.5G"`, what `lsblk` prints
/// without `--bytes`.
#[test]
fn a_size_in_another_shape_is_no_size_and_no_error() {
    let worded = LIMA_BTRFS.replace(
        &format!("\"size\": {LIMA_ROOT_SIZE},"),
        "\"size\": \"19.5G\",",
    );

    assert_ne!(worded, LIMA_BTRFS);
    let inventory = parse_lsblk(&worded).unwrap();

    assert_eq!(inventory.systems[0].size, 0);
    assert_eq!(inventory.disks.len(), 1);
}

/// Edit: the live ISO's empty `/dev/vda` with size 0, as a card reader
/// without a card shows.
#[test]
fn a_card_reader_without_a_card_is_no_disk() {
    let empty = LIVE.replace(&format!("\"size\": {LIVE_DISK_SIZE},"), "\"size\": 0,");

    assert_ne!(empty, LIVE);
    let inventory = parse_lsblk(&empty).unwrap();

    assert!(inventory.disks.is_empty(), "{:?}", inventory.disks);
}

/// The ISO's own EFI partition, its verity layers, the DVD and zram are
/// no place for a boot loader. Only the empty disk is.
#[test]
fn the_live_iso_offers_only_the_empty_disk() {
    let inventory = parse_lsblk(LIVE).unwrap();
    let disks: Vec<(&str, u64, bool)> = inventory
        .disks
        .iter()
        .map(|disk| (disk.device.as_str(), disk.size, disk.in_use))
        .collect();

    assert_eq!(disks, [("/dev/vda", LIVE_DISK_SIZE, false)]);
    assert!(inventory.esps.is_empty(), "{:?}", inventory.esps);
    assert!(inventory.systems.is_empty(), "{:?}", inventory.systems);
}

/// `/dev/vdb` of the Lima VM is LUKS on the whole disk with LVM inside,
/// no partition table.
#[test]
fn volumes_on_a_luks_disk_without_partitions_are_unlocked_systems() {
    let inventory = parse_lsblk(LIMA_LVM_ON_LUKS).unwrap();
    let systems: Vec<(&str, Encryption, bool)> = inventory
        .systems
        .iter()
        .map(|system| (system.device(), system.encryption, system.in_use))
        .collect();

    assert_eq!(
        systems,
        [
            ("/dev/vda2", Encryption::None, true),
            ("/dev/mapper/vg0-root", Encryption::Unlocked, true),
            ("/dev/mapper/vg0-home", Encryption::Unlocked, true),
        ]
    );
}
