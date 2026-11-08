use std::fs;

use toolbox_plan::procedure::Builtin;
use toolbox_plan::protocol::{Failure, Outcome};
use toolbox_plan::{
    BootloaderId, InstalledSystem, PartitionDevice, REPAIR_ROOT, RepairNeed, RepairRoot,
    RepairWork, RootLayout, VolumeDevice,
};

use super::{install, mounted, run};
use crate::repair::testing::Machine;
use crate::repair::{Lacking, installed_system};

#[test]
fn preparing_the_mount_is_one_step() {
    let machine = Machine::new();
    let (outcome, text) = run(machine.root(), &Builtin::PrepareRepairMount);

    assert_eq!(outcome, Outcome::Succeeded);
    assert!(text.contains("is ready"), "{text}");
}

#[test]
fn a_mount_left_from_an_earlier_repair_fails_its_step() {
    let mut machine = Machine::new();

    machine.mount("/dev/sda2", &format!("/{REPAIR_ROOT}"));

    let (outcome, text) = run(machine.root(), &Builtin::PrepareRepairMount);

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown
        }
    );
    assert!(text.contains("still mounted"), "{text}");
}

#[test]
fn requiring_a_system_names_what_it_lacks() {
    let machine = Machine::new();

    let (outcome, text) = run(
        machine.root(),
        &Builtin::RequireInstalledSystem(RepairNeed::Menu),
    );

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown
        }
    );
    assert!(text.contains("grub-mkconfig"), "{text}");

    install(machine.root());

    let (installed, _) = run(
        machine.root(),
        &Builtin::RequireInstalledSystem(RepairNeed::Menu),
    );

    assert_eq!(installed, Outcome::Succeeded);
}

#[test]
fn an_efi_system_elsewhere_is_no_uefi_repair() {
    let machine = Machine::new();
    let system = mounted(machine.root());

    fs::create_dir_all(system.join("usr/bin")).unwrap();
    fs::create_dir_all(system.join("boot")).unwrap();
    fs::write(system.join("usr/bin/grub-install"), "").unwrap();
    fs::write(system.join("usr/bin/grub-mkconfig"), "").unwrap();
    fs::write(system.join("boot/vmlinuz-linux"), "").unwrap();

    let (outcome, text) = run(
        machine.root(),
        &Builtin::RequireInstalledSystem(RepairNeed::InstallerWithEsp),
    );

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown
        }
    );
    assert!(text.contains("EFI system partition"), "{text}");
}

#[test]
fn an_unreadable_system_is_no_answer() {
    let machine = Machine::new();
    let system = mounted(machine.root());

    fs::create_dir_all(system.join("usr")).unwrap();
    fs::write(system.join("usr/bin"), "").unwrap();

    let (outcome, text) = run(
        machine.root(),
        &Builtin::RequireInstalledSystem(RepairNeed::Menu),
    );

    assert_eq!(
        outcome,
        Outcome::Failed {
            cause: Failure::Unknown
        }
    );
    assert!(text.contains("cannot look into"), "{text}");
}

#[test]
fn rechecking_without_the_devices_fails() {
    let root = tempfile::tempdir().unwrap();
    let system = InstalledSystem {
        root: RepairRoot::Open {
            device: VolumeDevice::new("/dev/sda2").unwrap(),
        },
        layout: RootLayout::Plain,
    };

    let work = RepairWork::InstallUefi {
        esp: PartitionDevice::new("/dev/sda1").unwrap(),
        id: BootloaderId::new("Garuda").unwrap(),
    };

    let (rechecked, _) = run(root.path(), &Builtin::RecheckRepairDevices { system, work });

    assert_eq!(
        rechecked,
        Outcome::Failed {
            cause: Failure::Unknown
        }
    );
}

#[test]
fn a_system_without_a_boot_folder_has_no_kernel() {
    let machine = Machine::new();
    let system = mounted(machine.root());

    fs::create_dir_all(system.join("usr/bin")).unwrap();
    fs::write(system.join("usr/bin/grub-mkconfig"), "").unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::Menu).unwrap(),
        Some(Lacking::Kernel)
    );
}
