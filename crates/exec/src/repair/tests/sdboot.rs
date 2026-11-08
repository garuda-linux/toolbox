//! An installed system that starts with systemd-boot, as the Garuda Lima
//! image lays it out.

use std::fs;
use std::path::Path;

use toolbox_plan::procedure::{Builtin, Program};
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{
    InstalledSystem, PartitionDevice, RepairMount, RepairNeed, RepairRoot, RepairWork, RootLayout,
    VolumeDevice,
};

use super::super::testing::Machine;
use super::super::{Lacking, boot_loader_steps, installed_system};
use super::{mounted, run};

/// `/etc/fstab` and `/etc/kernel/install.conf` of the Garuda Lima image.
const FSTAB: &str = include_str!("../../../../plan/src/repair/fixtures/sdboot-fstab");
const INSTALL_CONF: &str = include_str!("../../../../state/src/boot/fixtures/install.conf");

fn image(root: &Path) {
    let system = mounted(root);

    for folder in ["usr/bin", "usr/lib/garuda", "etc/kernel", "efi"] {
        fs::create_dir_all(system.join(folder)).unwrap();
    }

    for place in RepairMount::BINDS {
        fs::create_dir_all(system.join(place.inside())).unwrap();
    }

    fs::write(system.join("etc/fstab"), FSTAB).unwrap();
    fs::write(system.join("etc/kernel/install.conf"), INSTALL_CONF).unwrap();

    for program in [
        "usr/bin/bootctl",
        "usr/bin/kernel-install",
        "usr/lib/garuda/kernel-install-uki",
    ] {
        fs::write(system.join(program), "").unwrap();
    }
}

#[test]
fn the_image_passes_both_repairs_of_a_uefi_machine() {
    let machine = Machine::new();

    image(machine.root());

    for need in [RepairNeed::MenuWithEsp, RepairNeed::InstallerWithEsp] {
        assert_eq!(installed_system(machine.root(), need).unwrap(), None);
        assert_eq!(
            run(machine.root(), &Builtin::RequireInstalledSystem(need)).0,
            Outcome::Succeeded
        );
    }
}

/// Without an ESP the page names, the machine did not start through UEFI.
#[test]
fn without_uefi_systemd_boot_is_not_repaired() {
    let machine = Machine::new();

    image(machine.root());

    for need in [RepairNeed::Menu, RepairNeed::Installer] {
        assert_eq!(
            installed_system(machine.root(), need).unwrap(),
            Some(Lacking::Uefi)
        );
    }
}

#[test]
fn each_missing_part_is_named() {
    let machine = Machine::new();
    let system = mounted(machine.root());

    image(machine.root());
    fs::remove_file(system.join("usr/bin/bootctl")).unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::InstallerWithEsp).unwrap(),
        Some(Lacking::Program(Program::Bootctl))
    );
    assert_eq!(
        installed_system(machine.root(), RepairNeed::MenuWithEsp).unwrap(),
        None
    );

    fs::remove_file(system.join("usr/bin/kernel-install")).unwrap();
    fs::remove_file(system.join("usr/lib/garuda/kernel-install-uki")).unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::MenuWithEsp).unwrap(),
        Some(Lacking::Program(Program::KernelInstall))
    );
}

/// The image's ESP line commented out, then the folder it names gone.
#[test]
fn the_esp_comes_from_fstab_and_its_folder_must_exist() {
    let machine = Machine::new();
    let system = mounted(machine.root());

    image(machine.root());
    fs::write(
        system.join("etc/fstab"),
        FSTAB.replace("UUID=6438-BB38", "#UUID=6438-BB38"),
    )
    .unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::MenuWithEsp).unwrap(),
        Some(Lacking::EspInFstab)
    );

    fs::write(system.join("etc/fstab"), FSTAB).unwrap();
    fs::remove_dir(system.join("efi")).unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::MenuWithEsp).unwrap(),
        Some(Lacking::EspFolder)
    );
}

#[test]
fn the_steps_mount_the_esp_where_the_images_fstab_does() {
    let machine = Machine::new();

    image(machine.root());

    let system = InstalledSystem {
        root: RepairRoot::Open {
            device: VolumeDevice::new("/dev/vda2").unwrap(),
        },
        layout: RootLayout::Subvolume,
    };

    let work = RepairWork::RegenerateConfig {
        esp: Some(PartitionDevice::new("/dev/vda1").unwrap()),
    };

    let steps: Vec<String> = boot_loader_steps(machine.root(), &system, &work)
        .unwrap()
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(
        steps.first().map(String::as_str),
        Some("mount /dev/vda1 /run/garuda-toolbox/boot-repair/root/efi")
    );
}
