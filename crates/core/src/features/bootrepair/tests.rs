use toolbox_plan::{
    DiskDevice, PartitionDevice, Password, PlannedAction, RepairRoot, RepairWork, RootAction,
    RootLayout, Segment, VolumeDevice,
};

use super::{
    Encryption, Firmware, Inventory, Refusal, RepairFacts, SAMPLE_LISTING, Target, Tool, Wish,
    parse_lsblk, plan_regenerate, plan_reinstall,
};

const NVME: &str = include_str!("fixtures/nvme-ext4.json");
const LVM_ON_LUKS: &str = include_str!("fixtures/lvm-on-luks.json");
const INSTALLED: &str = include_str!("fixtures/installed-btrfs.json");

const PASSPHRASE: &str = "correct horse battery staple";

fn facts(listing: &str, firmware: Firmware) -> RepairFacts {
    RepairFacts {
        firmware,
        missing: Vec::new(),
        inventory: parse_lsblk(listing).unwrap(),
    }
}

fn wish(system: &str) -> Wish {
    Wish {
        system: VolumeDevice::new(system).unwrap(),
        layout: None,
        passphrase: None,
    }
}

fn locked_wish(system: &str, layout: Option<RootLayout>) -> Wish {
    Wish {
        system: VolumeDevice::new(system).unwrap(),
        layout,
        passphrase: Some(Password::new(PASSPHRASE.to_owned()).unwrap()),
    }
}

fn esp(device: &str) -> Target {
    Target::Esp(PartitionDevice::new(device).unwrap())
}

fn disk(device: &str) -> Target {
    Target::Disk(DiskDevice::new(device).unwrap())
}

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

fn action(plan: &toolbox_plan::Plan) -> RootAction {
    let actions: Vec<RootAction> = plan
        .clone()
        .into_segments()
        .into_iter()
        .flat_map(|segment| match segment {
            Segment::Root(steps) => steps.into_iter().map(|step| step.action).collect(),
            Segment::User(_) => Vec::new(),
        })
        .collect();

    assert_eq!(actions.len(), 1);
    actions.into_iter().next().unwrap()
}

#[test]
fn the_live_sample_lists_the_installed_disk_and_marks_the_medium() {
    let inventory = parse_lsblk(SAMPLE_LISTING).unwrap();

    assert_eq!(
        systems(&inventory),
        [
            ("/dev/sda2", "btrfs", Encryption::None, false),
            ("/dev/sda3", "", Encryption::Locked, false),
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
            ("/dev/sda1", "/dev/sda", false),
            ("/dev/sdb2", "/dev/sdb", true)
        ]
    );

    let disks: Vec<_> = inventory
        .disks
        .iter()
        .map(|disk| (disk.device.as_str(), disk.model.as_str(), disk.in_use))
        .collect();

    assert_eq!(
        disks,
        [
            ("/dev/sda", "Samsung SSD 870 EVO 500GB", false),
            ("/dev/sdb", "Ultra Fit", true),
        ]
    );
}

#[test]
fn nvme_and_card_names_are_read_and_swap_keeps_no_disk_busy() {
    let inventory = parse_lsblk(NVME).unwrap();

    assert_eq!(
        systems(&inventory),
        [("/dev/nvme0n1p2", "ext4", Encryption::None, false)]
    );
    assert_eq!(inventory.esps.len(), 1);

    let disks: Vec<_> = inventory
        .disks
        .iter()
        .map(|disk| (disk.device.as_str(), disk.in_use))
        .collect();

    assert_eq!(disks, [("/dev/nvme0n1", false), ("/dev/mmcblk0", false)]);
}

#[test]
fn volumes_on_an_unlocked_luks_partition_are_mapped_systems() {
    let inventory = parse_lsblk(LVM_ON_LUKS).unwrap();

    assert_eq!(
        systems(&inventory),
        [
            ("/dev/mapper/vg0-root", "ext4", Encryption::Unlocked, false),
            ("/dev/mapper/vg0-home", "ext4", Encryption::Unlocked, false),
        ]
    );

    let root = inventory.system("/dev/mapper/vg0-root").unwrap();

    assert_eq!(root.disks.len(), 1);
    assert_eq!(root.disks.first().unwrap().as_str(), "/dev/nvme0n1");
    assert!(!root.asks_layout());
    assert_eq!(
        inventory.usual_esp(root).unwrap().device.as_str(),
        "/dev/nvme0n1p1"
    );
}

#[test]
fn a_running_btrfs_system_with_subvolumes_is_in_use() {
    let inventory = parse_lsblk(INSTALLED).unwrap();

    assert_eq!(
        systems(&inventory),
        [
            ("/dev/vda2", "btrfs", Encryption::None, true),
            ("/dev/vdb1", "btrfs", Encryption::None, false),
        ]
    );
    assert!(inventory.esps.first().unwrap().in_use);
    assert!(inventory.system("/dev/vda2").unwrap().asks_layout());

    let other = inventory.system("/dev/vdb1").unwrap();

    assert_eq!(inventory.usual_esp(other), None);
    assert_eq!(
        inventory.usual_disk(other).unwrap().device.as_str(),
        "/dev/vdb"
    );
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

#[test]
fn a_uefi_reinstall_takes_the_system_s_layout_and_the_named_esp() {
    let facts = facts(SAMPLE_LISTING, Firmware::Uefi);
    let plan = plan_reinstall(&facts, wish("/dev/sda2"), &esp("/dev/sda1")).unwrap();

    let RootAction::RepairBoot {
        system,
        work,
        passphrase,
    } = action(&plan)
    else {
        panic!("not a repair");
    };

    assert_eq!(system.layout, RootLayout::Subvolume);
    assert!(matches!(&system.root, RepairRoot::Open { device } if device.as_str() == "/dev/sda2"));
    assert!(matches!(&work, RepairWork::InstallUefi { esp, id }
        if esp.as_str() == "/dev/sda1" && id.as_str() == "Garuda"));
    assert_eq!(passphrase, None);
}

#[test]
fn a_bios_reinstall_takes_a_whole_disk_of_the_listing() {
    let facts = facts(NVME, Firmware::Bios);
    let plan = plan_reinstall(&facts, wish("/dev/nvme0n1p2"), &disk("/dev/nvme0n1")).unwrap();

    let RootAction::RepairBoot {
        system,
        work,
        passphrase: _,
    } = action(&plan)
    else {
        panic!("not a repair");
    };

    assert_eq!(system.layout, RootLayout::Plain);
    assert!(matches!(&work, RepairWork::InstallBios { disk } if disk.as_str() == "/dev/nvme0n1"));

    assert_eq!(
        plan_reinstall(&facts, wish("/dev/nvme0n1p2"), &esp("/dev/nvme0n1p1")).unwrap_err(),
        Refusal::UnknownTarget
    );
}

#[test]
fn the_target_must_fit_how_the_machine_started() {
    let uefi = facts(SAMPLE_LISTING, Firmware::Uefi);
    let bios = facts(SAMPLE_LISTING, Firmware::Bios);

    assert_eq!(
        plan_reinstall(&uefi, wish("/dev/sda2"), &disk("/dev/sda")).unwrap_err(),
        Refusal::UnknownTarget
    );
    assert_eq!(
        plan_reinstall(&uefi, wish("/dev/sda2"), &esp("/dev/sdz1")).unwrap_err(),
        Refusal::UnknownTarget
    );
    assert_eq!(
        plan_reinstall(&bios, wish("/dev/sda2"), &esp("/dev/sda1")).unwrap_err(),
        Refusal::UnknownTarget
    );
}

#[test]
fn nothing_on_a_disk_in_use_is_repaired_or_written() {
    let live = facts(SAMPLE_LISTING, Firmware::Uefi);
    let bios = facts(SAMPLE_LISTING, Firmware::Bios);
    let installed = facts(INSTALLED, Firmware::Uefi);

    assert_eq!(
        plan_reinstall(&live, wish("/dev/sda2"), &esp("/dev/sdb2")).unwrap_err(),
        Refusal::InUse
    );
    assert_eq!(
        plan_reinstall(&bios, wish("/dev/sda2"), &disk("/dev/sdb")).unwrap_err(),
        Refusal::InUse
    );
    assert_eq!(
        plan_regenerate(&installed, wish("/dev/vda2")).unwrap_err(),
        Refusal::InUse
    );
}

#[test]
fn only_a_listed_system_is_repaired() {
    let facts = facts(SAMPLE_LISTING, Firmware::Uefi);

    for system in ["/dev/sda1", "/dev/sdb1", "/dev/sdz2", "/dev/mapper/other"] {
        assert_eq!(
            plan_regenerate(&facts, wish(system)).unwrap_err(),
            Refusal::UnknownSystem,
            "{system:?}"
        );
    }
}

#[test]
fn a_locked_system_needs_a_passphrase_and_takes_the_chosen_layout() {
    let facts = facts(SAMPLE_LISTING, Firmware::Uefi);

    assert_eq!(
        plan_regenerate(&facts, wish("/dev/sda3"))
            .unwrap_err()
            .as_str(),
        "empty"
    );
    for (layout, expected) in [
        (None, RootLayout::Subvolume),
        (Some(RootLayout::Subvolume), RootLayout::Subvolume),
        (Some(RootLayout::Plain), RootLayout::Plain),
    ] {
        let plan = plan_regenerate(&facts, locked_wish("/dev/sda3", layout)).unwrap();
        let repair = action(&plan);

        assert!(repair.needs_secret());
        assert!(repair.secret().is_some());
        assert!(
            matches!(&repair, RootAction::RepairBoot { system, work: _, passphrase: _ }
            if system.layout == expected && system.is_locked())
        );
        assert!(!format!("{plan:?}").contains("horse"));
    }
}

#[test]
fn an_open_system_drops_a_passphrase_and_a_layout_it_cannot_have() {
    let facts = facts(NVME, Firmware::Uefi);
    let plan = plan_regenerate(
        &facts,
        locked_wish("/dev/nvme0n1p2", Some(RootLayout::Subvolume)),
    )
    .unwrap();

    let repair = action(&plan);

    assert!(!repair.needs_secret());
    assert!(
        matches!(&repair, RootAction::RepairBoot { system, work: _, passphrase: None }
        if system.layout == RootLayout::Plain)
    );
}

#[test]
fn a_missing_program_stops_every_repair() {
    let mut facts = facts(SAMPLE_LISTING, Firmware::Uefi);

    facts.missing = vec![Tool::Cryptsetup];

    assert_eq!(
        plan_regenerate(&facts, wish("/dev/sda2")).unwrap_err(),
        Refusal::Unavailable
    );
    assert_eq!(
        plan_reinstall(&facts, wish("/dev/sda2"), &esp("/dev/sda1")).unwrap_err(),
        Refusal::Unavailable
    );
}

#[test]
fn a_plan_is_one_root_step() {
    let facts = facts(SAMPLE_LISTING, Firmware::Uefi);
    let plan = plan_regenerate(&facts, wish("/dev/sda2")).unwrap();

    assert_eq!(plan.steps().len(), 1);
    assert!(matches!(
        plan.steps().first().map(|step| &step.action),
        Some(PlannedAction::Root(RootAction::RepairBoot {
            system: _,
            work: RepairWork::RegenerateConfig,
            passphrase: None
        }))
    ));
}

#[test]
fn tools_are_named_by_their_program() {
    let names: Vec<_> = Tool::ALL.into_iter().map(Tool::name).collect();

    assert_eq!(names, ["lsblk", "grub-install", "cryptsetup"]);
}

#[test]
fn the_boot_entry_keeps_the_name_of_the_installer() {
    assert_eq!(super::BOOTLOADER_ID.as_str(), "Garuda");
}
