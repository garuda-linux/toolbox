mod execute;

use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::{Path, PathBuf};

use toolbox_plan::procedure::{Builtin, Program};
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{REPAIR_ROOT, RepairMount, RepairNeed};

use super::testing::Machine;
use super::{Lacking, installed_system, prepare_mount};
use crate::testing::run_builtin as run;

fn mounted(root: &Path) -> PathBuf {
    root.join(REPAIR_ROOT)
}

fn install(root: &Path) {
    let system = mounted(root);

    fs::create_dir_all(system.join("usr/bin")).unwrap();
    fs::create_dir_all(system.join("boot/efi")).unwrap();
    fs::create_dir_all(system.join("boot/grub")).unwrap();

    for place in RepairMount::BINDS {
        fs::create_dir_all(system.join(place.inside())).unwrap();
    }

    fs::write(system.join("usr/bin/grub-install"), "").unwrap();
    fs::write(system.join("usr/bin/grub-mkconfig"), "").unwrap();
    fs::write(system.join("boot/vmlinuz-linux-zen"), "").unwrap();
}

#[test]
fn the_mount_point_is_made_in_a_folder_for_root_alone() {
    let machine = Machine::new();

    prepare_mount(machine.root()).unwrap();

    let folder = mounted(machine.root());
    let private = folder.parent().unwrap();

    assert!(folder.is_dir());
    assert_eq!(
        fs::metadata(private).unwrap().permissions().mode() & 0o777,
        0o700
    );

    prepare_mount(machine.root()).unwrap();
}

#[test]
fn a_symlink_on_the_way_to_the_mount_point_is_refused() {
    let machine = Machine::new();
    let elsewhere = machine.root().join("elsewhere");

    fs::create_dir_all(&elsewhere).unwrap();
    symlink(&elsewhere, machine.root().join("run/garuda-toolbox")).unwrap();

    assert!(prepare_mount(machine.root()).is_err());
    assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 0);
}

#[test]
fn an_installed_system_with_grub_passes_every_need() {
    let machine = Machine::new();

    install(machine.root());

    for need in [
        RepairNeed::Menu,
        RepairNeed::Installer,
        RepairNeed::InstallerWithEsp,
    ] {
        assert_eq!(installed_system(machine.root(), need).unwrap(), None);
        assert_eq!(
            run(machine.root(), &Builtin::RequireInstalledSystem(need)).0,
            Outcome::Succeeded
        );
    }
}

/// The top of a btrfs whose system lies in `@`, a data partition, an
/// empty one: none has `/usr/bin`.
#[test]
fn a_partition_without_grub_is_no_system_to_repair() {
    let machine = Machine::new();

    prepare_mount(machine.root()).unwrap();
    assert_eq!(
        installed_system(machine.root(), RepairNeed::Menu).unwrap(),
        Some(Lacking::Program(Program::GrubMkconfig))
    );

    install(machine.root());
    fs::remove_file(mounted(machine.root()).join("usr/bin/grub-install")).unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::Menu).unwrap(),
        None
    );
    assert_eq!(
        installed_system(machine.root(), RepairNeed::Installer).unwrap(),
        Some(Lacking::Program(Program::GrubInstall))
    );

    let (outcome, text) = run(
        machine.root(),
        &Builtin::RequireInstalledSystem(RepairNeed::Installer),
    );

    assert!(matches!(outcome, Outcome::Failed { cause: _ }));
    assert!(text.contains("no /usr/bin/grub-install"), "{text}");
}

#[test]
fn a_program_that_is_a_symlink_does_not_count() {
    let machine = Machine::new();

    install(machine.root());

    let program = mounted(machine.root()).join("usr/bin/grub-mkconfig");

    fs::remove_file(&program).unwrap();
    symlink("/usr/bin/true", &program).unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::Menu).unwrap(),
        Some(Lacking::Program(Program::GrubMkconfig))
    );
}

#[test]
fn a_boot_folder_without_a_kernel_means_a_separate_boot_partition() {
    let machine = Machine::new();

    install(machine.root());
    fs::remove_file(mounted(machine.root()).join("boot/vmlinuz-linux-zen")).unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::Menu).unwrap(),
        Some(Lacking::Kernel)
    );

    let (_, text) = run(
        machine.root(),
        &Builtin::RequireInstalledSystem(RepairNeed::Menu),
    );

    assert!(text.contains("no kernel in /boot"), "{text}");
}

#[test]
fn a_uefi_install_needs_the_folder_of_the_efi_partition() {
    let machine = Machine::new();

    install(machine.root());
    fs::remove_dir(mounted(machine.root()).join("boot/efi")).unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::Installer).unwrap(),
        None
    );
    assert_eq!(
        installed_system(machine.root(), RepairNeed::InstallerWithEsp).unwrap(),
        Some(Lacking::EspFolder)
    );
}

#[test]
fn a_bind_target_that_is_a_symlink_stops_the_repair() {
    let machine = Machine::new();

    install(machine.root());
    fs::remove_dir(mounted(machine.root()).join("dev")).unwrap();
    symlink("/usr/bin", mounted(machine.root()).join("dev")).unwrap();

    assert_eq!(
        installed_system(machine.root(), RepairNeed::Menu).unwrap(),
        Some(Lacking::BindTarget(RepairMount::Dev))
    );

    let (_, text) = run(
        machine.root(),
        &Builtin::RequireInstalledSystem(RepairNeed::Menu),
    );

    assert!(text.contains("/dev is missing or a symlink"), "{text}");
}

#[test]
fn the_checks_never_write_into_the_mounted_system() {
    let machine = Machine::new();

    install(machine.root());

    let before = listing(&mounted(machine.root()));

    for need in [RepairNeed::Menu, RepairNeed::InstallerWithEsp] {
        installed_system(machine.root(), need).unwrap();
    }

    assert_eq!(listing(&mounted(machine.root())), before);
}

fn listing(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![folder.to_owned()];

    while let Some(next) = pending.pop() {
        for entry in fs::read_dir(&next).unwrap() {
            let path = entry.unwrap().path();

            if path.is_dir() {
                pending.push(path.clone());
            }

            found.push(path);
        }
    }

    found.sort();
    found
}
