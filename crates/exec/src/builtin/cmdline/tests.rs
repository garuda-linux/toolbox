use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use toolbox_plan::KernelParameters;

use super::write_kernel_cmdline;
use crate::builtin::FileChange;

/// `/etc/kernel/cmdline` of the Garuda Lima image.
const CMDLINE: &str = include_str!("../../../../state/src/boot/fixtures/kernel-cmdline");

fn image(root: &Path) {
    fs::create_dir_all(root.join("etc/kernel")).unwrap();
    fs::write(root.join("etc/kernel/cmdline"), CMDLINE).unwrap();
}

fn without_quiet() -> KernelParameters {
    KernelParameters::parse(CMDLINE).unwrap().without("quiet")
}

fn read(root: &Path, name: &str) -> String {
    fs::read_to_string(root.join("etc/kernel").join(name)).unwrap()
}

#[test]
fn the_images_file_gets_the_list_and_keeps_the_old_one_beside_it() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());

    let change = write_kernel_cmdline(dir.path(), &without_quiet()).unwrap();

    assert_eq!(change, FileChange::Written);
    assert_eq!(
        read(dir.path(), "cmdline"),
        "root=UUID=cd2fff21-4bb7-40f3-a158-406f545e11e6 rootflags=subvol=@ rw loglevel=3 console=tty0 console=ttyS0,115200\n"
    );
    assert_eq!(read(dir.path(), "cmdline.bak"), CMDLINE);
}

/// kernel-install falls back to `/proc/cmdline` without the file, so the
/// page's list starts from there and the first write makes the file.
#[test]
fn a_missing_file_is_created_without_a_backup() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("etc/kernel")).unwrap();

    write_kernel_cmdline(dir.path(), &without_quiet()).unwrap();

    assert!(read(dir.path(), "cmdline").starts_with("root=UUID="));
    assert!(!dir.path().join("etc/kernel/cmdline.bak").exists());
}

#[test]
fn the_same_list_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());

    assert_eq!(
        write_kernel_cmdline(dir.path(), &KernelParameters::parse(CMDLINE).unwrap()).unwrap(),
        FileChange::Unchanged
    );
    assert!(!dir.path().join("etc/kernel/cmdline.bak").exists());
}

#[test]
fn a_symlinked_file_is_not_followed() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("etc/kernel")).unwrap();
    fs::write(dir.path().join("target"), CMDLINE).unwrap();
    symlink(
        dir.path().join("target"),
        dir.path().join("etc/kernel/cmdline"),
    )
    .unwrap();

    write_kernel_cmdline(dir.path(), &without_quiet()).unwrap_err();
    assert_eq!(
        fs::read_to_string(dir.path().join("target")).unwrap(),
        CMDLINE
    );
}
