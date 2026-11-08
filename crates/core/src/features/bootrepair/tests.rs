mod inventory;
pub(super) mod listings;

use listings::{BARE_METAL, INSTALLED, laptop_from_the_live_iso, lvm_with_only_swap};
use toolbox_plan::{
    DiskDevice, PartitionDevice, Password, PlannedAction, RepairRoot, RepairWork, RootAction,
    RootLayout, Segment, VolumeDevice,
};

use super::{
    Firmware, Refusal, RepairFacts, Target, Tool, Wish, parse_lsblk, plan_regenerate,
    plan_reinstall,
};

const PASSPHRASE: &str = "correct horse battery staple";

/// The free second Garuda of the laptop and the EFI partition on its disk.
const OTHER: &str = "/dev/nvme1n1p2";
const OTHER_ESP: &str = "/dev/nvme1n1p1";

pub(super) fn facts(listing: &str, firmware: Firmware) -> RepairFacts {
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

pub(super) fn esp(device: &str) -> Target {
    Target::Esp(PartitionDevice::new(device).unwrap())
}

fn disk(device: &str) -> Target {
    Target::Disk(DiskDevice::new(device).unwrap())
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
fn a_uefi_reinstall_takes_the_system_s_layout_and_the_named_esp() {
    let facts = facts(BARE_METAL, Firmware::Uefi);
    let plan = plan_reinstall(&facts, wish(OTHER), &esp(OTHER_ESP)).unwrap();

    let RootAction::RepairBoot {
        system,
        work,
        passphrase,
    } = action(&plan)
    else {
        panic!("not a repair");
    };

    assert_eq!(system.layout, RootLayout::Subvolume);
    assert!(matches!(&system.root, RepairRoot::Open { device } if device.as_str() == OTHER));
    assert!(matches!(&work, RepairWork::InstallUefi { esp, id }
        if esp.as_str() == OTHER_ESP && id.as_str() == "Garuda"));
    assert_eq!(passphrase, None);
}

#[test]
fn a_bios_reinstall_takes_a_whole_disk_of_the_listing() {
    let facts = facts(&lvm_with_only_swap(), Firmware::Bios);
    let root = "/dev/mapper/vg0-root";
    let plan = plan_reinstall(&facts, wish(root), &disk("/dev/vdb")).unwrap();

    let RootAction::RepairBoot {
        system,
        work,
        passphrase: _,
    } = action(&plan)
    else {
        panic!("not a repair");
    };

    assert_eq!(system.layout, RootLayout::Plain);
    assert!(matches!(&work, RepairWork::InstallBios { disk } if disk.as_str() == "/dev/vdb"));

    assert_eq!(
        plan_reinstall(&facts, wish(root), &esp("/dev/vda1")).unwrap_err(),
        Refusal::UnknownTarget
    );
}

#[test]
fn the_target_must_fit_how_the_machine_started() {
    let uefi = facts(BARE_METAL, Firmware::Uefi);
    let bios = facts(BARE_METAL, Firmware::Bios);

    assert_eq!(
        plan_reinstall(&uefi, wish(OTHER), &disk("/dev/nvme1n1")).unwrap_err(),
        Refusal::UnknownTarget
    );
    assert_eq!(
        plan_reinstall(&uefi, wish(OTHER), &esp("/dev/sdz1")).unwrap_err(),
        Refusal::UnknownTarget
    );
    assert_eq!(
        plan_reinstall(&bios, wish(OTHER), &esp(OTHER_ESP)).unwrap_err(),
        Refusal::UnknownTarget
    );
}

#[test]
fn nothing_on_a_disk_in_use_is_repaired_or_written() {
    let uefi = facts(BARE_METAL, Firmware::Uefi);
    let bios = facts(BARE_METAL, Firmware::Bios);
    let installed = facts(INSTALLED, Firmware::Uefi);

    assert_eq!(
        plan_reinstall(&uefi, wish(OTHER), &esp("/dev/nvme0n1p2")).unwrap_err(),
        Refusal::InUse
    );
    assert_eq!(
        plan_reinstall(&bios, wish(OTHER), &disk("/dev/nvme0n1")).unwrap_err(),
        Refusal::InUse
    );
    assert_eq!(
        plan_regenerate(&uefi, wish("/dev/mapper/garuda")).unwrap_err(),
        Refusal::InUse
    );
    assert_eq!(
        plan_regenerate(&installed, wish("/dev/vda2")).unwrap_err(),
        Refusal::InUse
    );
}

#[test]
fn only_a_listed_system_is_repaired() {
    let facts = facts(BARE_METAL, Firmware::Uefi);

    for system in [
        OTHER_ESP,
        "/dev/nvme0n1p3",
        "/dev/sda1",
        "/dev/sdz2",
        "/dev/mapper/other",
    ] {
        assert_eq!(
            plan_regenerate(&facts, wish(system)).unwrap_err(),
            Refusal::UnknownSystem,
            "{system:?}"
        );
    }
}

#[test]
fn a_locked_system_needs_a_passphrase_and_takes_the_chosen_layout() {
    let facts = facts(&laptop_from_the_live_iso(), Firmware::Uefi);
    let locked = "/dev/nvme0n1p1";

    assert_eq!(
        plan_regenerate(&facts, wish(locked)).unwrap_err().as_str(),
        "empty"
    );
    for (layout, expected) in [
        (None, RootLayout::Subvolume),
        (Some(RootLayout::Subvolume), RootLayout::Subvolume),
        (Some(RootLayout::Plain), RootLayout::Plain),
    ] {
        let plan = plan_regenerate(&facts, locked_wish(locked, layout)).unwrap();
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
    let facts = facts(&lvm_with_only_swap(), Firmware::Uefi);
    let plan = plan_regenerate(
        &facts,
        locked_wish("/dev/mapper/vg0-root", Some(RootLayout::Subvolume)),
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
    let mut facts = facts(BARE_METAL, Firmware::Uefi);

    facts.missing = vec![Tool::Cryptsetup];

    assert_eq!(
        plan_regenerate(&facts, wish(OTHER)).unwrap_err(),
        Refusal::Unavailable
    );
    assert_eq!(
        plan_reinstall(&facts, wish(OTHER), &esp(OTHER_ESP)).unwrap_err(),
        Refusal::Unavailable
    );
}

#[test]
fn a_plan_is_one_root_step() {
    let facts = facts(BARE_METAL, Firmware::Uefi);
    let plan = plan_regenerate(&facts, wish(OTHER)).unwrap();

    assert_eq!(plan.steps().len(), 1);
    assert!(matches!(
        plan.steps().first().map(|step| &step.action),
        Some(PlannedAction::Root(RootAction::RepairBoot {
            system: _,
            work: RepairWork::RegenerateConfig { esp: None },
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
