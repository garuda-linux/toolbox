use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_plan::protocol::ReadKind;
use toolbox_state::boot::BootLoader;

use super::loader;
use crate::test_support::write;
use crate::{
    BootMenuSource as _, BootSource as _, HelperReads, MenuRead, ProbeError, SystemBoot,
    SystemBootMenu,
};

/// Files of the Garuda Lima image, and the `/etc/default/grub` of a Garuda
/// GRUB install.
const LOADER_INFO: &[u8] = toolbox_state::testing::boot::LOADERINFO.as_bytes();
const KERNEL_CMDLINE: &str = toolbox_state::testing::boot::KERNEL_CMDLINE;
const PROC_CMDLINE: &str = toolbox_plan::testing::host::PROC_CMDLINE;
const MOUNTINFO: &str = toolbox_plan::testing::sdboot::MOUNTINFO;
const UKI: &[u8] = toolbox_plan::testing::sdboot::GARUDA_7_3_1_1_GARUDA_EFI_HEAD;
const GRUB_DEFAULTS: &str = toolbox_plan::testing::grub::INSTALLED_DEFAULT_GRUB;

const LOADER_INFO_PATH: &str =
    "sys/firmware/efi/efivars/LoaderInfo-4a67b082-0a4c-41cf-b6c7-440b29bb8c4f";

struct NoHelper;

impl HelperReads for NoHelper {
    fn read_command(&self, kind: ReadKind) -> Command {
        panic!("asked the helper for {}", kind.as_str());
    }
}

fn boot(root: &Path) -> SystemBoot {
    SystemBoot::new(root, "/nonexistent/systemd-detect-virt")
}

#[test]
fn the_image_is_a_systemd_boot_system() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("sys/firmware/efi/efivars")).unwrap();
    std::fs::write(dir.path().join(LOADER_INFO_PATH), LOADER_INFO).unwrap();

    assert_eq!(loader(dir.path()), Ok(BootLoader::SystemdBoot));
}

#[test]
fn a_garuda_grub_install_is_a_grub_system() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "etc/default/grub", GRUB_DEFAULTS);

    assert_eq!(loader(dir.path()), Ok(BootLoader::Grub));
}

#[test]
fn the_images_command_line_is_read_word_by_word() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "etc/kernel/cmdline", KERNEL_CMDLINE);

    let cmdline = boot(dir.path()).kernel_cmdline().unwrap();
    let parameters = cmdline.parameters.unwrap();

    assert!(parameters.contains("quiet"));
    assert!(parameters.contains("rootflags=subvol=@"));
    assert_eq!(parameters.iter().count(), 7);
    assert!(!cmdline.running);
}

/// kernel-install takes the running command line without the file.
#[test]
fn without_the_file_the_running_command_line_counts() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "proc/cmdline", PROC_CMDLINE);
    write(dir.path(), "usr/bin/kernel-install", "");

    let cmdline = boot(dir.path()).kernel_cmdline().unwrap();

    assert!(cmdline.running);
    assert_eq!(cmdline.line, PROC_CMDLINE.trim_end());
}

/// NixOS writes its own entries: nothing would read a new file.
#[test]
fn without_kernel_install_the_running_command_line_is_no_file_to_change() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "proc/cmdline", PROC_CMDLINE);

    assert!(matches!(
        boot(dir.path()).kernel_cmdline(),
        Err(ProbeError::Unavailable)
    ));
}

#[test]
fn without_either_the_command_line_is_unavailable() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "usr/bin/kernel-install", "");

    assert!(matches!(
        boot(dir.path()).kernel_cmdline(),
        Err(ProbeError::Unavailable)
    ));
}

/// Where the user may read the ESP, the app reads it itself.
#[test]
fn a_readable_esp_is_read_without_the_helper() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("sys/firmware/efi/efivars")).unwrap();
    std::fs::write(dir.path().join(LOADER_INFO_PATH), LOADER_INFO).unwrap();
    write(dir.path(), "proc/self/mountinfo", MOUNTINFO);
    std::fs::create_dir_all(dir.path().join("efi/EFI/Linux")).unwrap();
    std::fs::write(
        dir.path().join("efi/EFI/Linux/garuda-7.3.1-1-garuda.efi"),
        UKI,
    )
    .unwrap();

    let source = SystemBootMenu::new(dir.path(), Arc::new(NoHelper));
    let read = source.menu(&Arc::new(AtomicBool::new(false)));

    let MenuRead::SdBoot(menu) = read else {
        panic!("{read:?}");
    };

    assert_eq!(menu.entries[0].title, "Garuda Linux");
}

/// `/etc/kernel/install.conf` is a folder, so it cannot be read
/// and the loader stays unknown instead of falling back to GRUB.
#[test]
fn an_unreadable_install_conf_fails_the_loader_read() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "etc/default/grub", GRUB_DEFAULTS);
    std::fs::create_dir_all(dir.path().join("etc/kernel/install.conf")).unwrap();

    assert!(matches!(
        loader(dir.path()),
        Err(ProbeError::Io { what, error: _ }) if what.contains("install.conf")
    ));
}
