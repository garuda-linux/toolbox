use std::fs;
use std::path::Path;

use toolbox_state::bootrepair::{Firmware, SAMPLE_LISTING, Tool};

use super::{DryRunRepair, RepairSource as _, SystemRepair};
use crate::error::ProbeError;
use crate::testing::fake_program;

fn install(root: &Path, tools: &[Tool]) {
    for tool in tools {
        let path = root.join(tool.path().trim_start_matches('/'));

        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }
}

fn source(root: &Path, lsblk: &Path) -> SystemRepair {
    SystemRepair {
        root: root.to_owned(),
        lsblk: lsblk.into(),
    }
}

#[test]
fn the_disks_come_from_lsblk_and_the_firmware_from_sys() {
    let root = tempfile::tempdir().unwrap();
    let (_folder, lsblk) = fake_program(SAMPLE_LISTING, "", 0);

    install(root.path(), &Tool::ALL);

    let bios = source(root.path(), &lsblk).facts().unwrap();

    assert_eq!(bios.firmware, Firmware::Bios);
    assert_eq!(bios.missing, []);
    assert_eq!(bios.inventory.systems.len(), 2);

    fs::create_dir_all(root.path().join("sys/firmware/efi")).unwrap();

    assert_eq!(
        source(root.path(), &lsblk).facts().unwrap().firmware,
        Firmware::Uefi
    );
}

#[test]
fn missing_programs_are_named_and_no_disk_is_read() {
    let root = tempfile::tempdir().unwrap();
    let (_folder, lsblk) = fake_program(SAMPLE_LISTING, "", 0);

    install(root.path(), &[Tool::Lsblk]);

    let facts = source(root.path(), &lsblk).facts().unwrap();

    assert_eq!(facts.missing, [Tool::GrubInstall, Tool::Cryptsetup]);
    assert_eq!(facts.inventory.systems, []);
    assert_eq!(facts.inventory.disks, []);
}

#[test]
fn a_failing_or_babbling_lsblk_is_a_failed_probe() {
    let root = tempfile::tempdir().unwrap();

    install(root.path(), &Tool::ALL);

    let (_failing, failing) = fake_program("", "lsblk: failed to access sysfs", 32);
    let (_babbling, babbling) = fake_program("no json", "", 0);

    assert!(matches!(
        source(root.path(), &failing).facts(),
        Err(ProbeError::Failed(reason)) if reason.contains("failed to access sysfs")
    ));
    assert!(matches!(
        source(root.path(), &babbling).facts(),
        Err(ProbeError::Failed(reason)) if reason.contains("no device list")
    ));
}

#[test]
fn the_dry_run_shows_a_made_up_uefi_machine() {
    let facts = DryRunRepair.facts().unwrap();

    assert_eq!(facts.firmware, Firmware::Uefi);
    assert_eq!(facts.missing, []);
    assert_eq!(
        facts.inventory.systems.len(),
        2,
        "an open and a locked system"
    );
}
