use super::{
    InstalledSystem, REPAIR_MAPPER, REPAIR_ROOT, REPAIR_VOLUME, RepairMount, RepairNeed,
    RepairRoot, RepairWork, RootLayout,
};
use crate::{BootloaderId, DiskDevice, PartitionDevice, VolumeDevice};

#[test]
fn every_mount_point_lies_below_the_helper_s_folder() {
    for place in RepairMount::ALL {
        let path = place.to_string();

        assert!(
            path == format!("/{REPAIR_ROOT}") || path.starts_with(&format!("/{REPAIR_ROOT}/")),
            "{path}"
        );
        assert!(
            path.split('/').all(|part| part != ".." && part != "."),
            "{path}"
        );
    }
}

#[test]
fn the_mount_points_are_the_usual_places_of_a_chroot() {
    let paths: Vec<String> = RepairMount::ALL
        .into_iter()
        .map(|place| place.to_string())
        .collect();

    assert_eq!(
        paths,
        [
            "/run/garuda-toolbox/boot-repair/root",
            "/run/garuda-toolbox/boot-repair/root/dev",
            "/run/garuda-toolbox/boot-repair/root/proc",
            "/run/garuda-toolbox/boot-repair/root/sys",
            "/run/garuda-toolbox/boot-repair/root/run",
            "/run/garuda-toolbox/boot-repair/root/sys/firmware/efi/efivars",
            "/run/garuda-toolbox/boot-repair/root/efi",
            "/run/garuda-toolbox/boot-repair/root/boot",
            "/run/garuda-toolbox/boot-repair/root/boot/efi",
        ]
    );
}

#[test]
fn the_repair_volume_is_the_mapper_below_dev_mapper() {
    assert_eq!(
        VolumeDevice::new(REPAIR_VOLUME).unwrap().as_str(),
        format!("/dev/mapper/{REPAIR_MAPPER}")
    );
}

#[test]
fn only_a_locked_root_takes_a_passphrase() {
    let locked = InstalledSystem {
        root: RepairRoot::Locked {
            partition: PartitionDevice::new("/dev/sda2").unwrap(),
        },
        layout: RootLayout::Subvolume,
    };

    let open = InstalledSystem {
        root: RepairRoot::Open {
            device: VolumeDevice::new("/dev/sda2").unwrap(),
        },
        layout: RootLayout::Plain,
    };

    assert!(locked.is_locked());
    assert!(!open.is_locked());
}

#[test]
fn each_kind_of_work_names_what_the_system_must_hold() {
    let uefi = RepairWork::InstallUefi {
        esp: PartitionDevice::new("/dev/sda1").unwrap(),
        id: BootloaderId::new("Garuda").unwrap(),
    };

    let bios = RepairWork::InstallBios {
        disk: DiskDevice::new("/dev/sda").unwrap(),
    };

    let regenerate = RepairWork::RegenerateConfig { esp: None };
    let regenerate_uefi = RepairWork::RegenerateConfig {
        esp: Some(PartitionDevice::new("/dev/sda1").unwrap()),
    };

    assert_eq!(regenerate.need(), RepairNeed::Menu);
    assert_eq!(regenerate_uefi.need(), RepairNeed::MenuWithEsp);
    assert_eq!(uefi.need(), RepairNeed::InstallerWithEsp);
    assert_eq!(bios.need(), RepairNeed::Installer);
}

#[test]
fn layouts_have_stable_ids() {
    assert_eq!(
        serde_json::to_string(&RootLayout::Subvolume).unwrap(),
        "\"subvolume\""
    );
}

/// The roles are types: the wire cannot put a whole disk where a
/// partition belongs, or a partition where the boot sector of a disk is
/// written.
#[test]
fn a_device_of_the_wrong_kind_does_not_decode() {
    for work in [
        serde_json::json!({ "kind": "install-bios", "disk": "/dev/sda1" }),
        serde_json::json!({ "kind": "install-bios", "disk": "/dev/mapper/vg-root" }),
        serde_json::json!({ "kind": "install-uefi", "esp": "/dev/sda", "id": "Garuda" }),
        serde_json::json!({ "kind": "install-uefi", "esp": "/dev/mapper/x", "id": "Garuda" }),
        serde_json::json!({ "kind": "install-uefi", "esp": "/dev/sda1", "id": "--removable" }),
        serde_json::json!({ "kind": "install-uefi", "esp": "/dev/sda1", "id": "../x" }),
        serde_json::json!({ "kind": "install-anywhere", "disk": "/dev/sda" }),
    ] {
        assert!(
            serde_json::from_value::<RepairWork>(work.clone()).is_err(),
            "{work}"
        );
    }

    for root in [
        serde_json::json!({ "kind": "locked", "partition": "/dev/sda" }),
        serde_json::json!({ "kind": "locked", "partition": "/dev/mapper/vg-root" }),
        serde_json::json!({ "kind": "open", "device": "/dev/sda" }),
        serde_json::json!({ "kind": "open", "device": "/mnt/disk.img" }),
    ] {
        assert!(
            serde_json::from_value::<RepairRoot>(root.clone()).is_err(),
            "{root}"
        );
    }
}
