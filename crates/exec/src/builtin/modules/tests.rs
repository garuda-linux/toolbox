use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use toolbox_plan::KernelModule;

use super::edit_modprobe_blacklist;
use crate::builtin::FileChange;
use crate::test_support::mode;

const FILE: &str = "etc/modprobe.d/garuda-toolbox-blacklist.conf";

/// `/usr/lib/modprobe.d/systemd.conf` of the Lima Garuda image.
const SYSTEMD_CONF: &str =
    toolbox_plan::testing::network::modules::ROOT_USR_LIB_MODPROBE_D_SYSTEMD_CONF;

fn module(name: &str) -> KernelModule {
    KernelModule::new(name).unwrap()
}

fn file(root: &Path) -> PathBuf {
    root.join(FILE)
}

#[test]
fn the_first_entry_creates_folder_and_file_world_readable() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        edit_modprobe_blacklist(root.path(), &module("r8169"), true).unwrap(),
        FileChange::Written
    );
    assert_eq!(
        fs::read_to_string(file(root.path())).unwrap(),
        "# Written by Garuda Toolbox (Network, Drivers). Edit it there.\nblacklist r8169\n"
    );
    assert_eq!(mode(&file(root.path())), 0o644);
}

#[test]
fn a_second_entry_keeps_the_first_and_a_repeat_changes_nothing() {
    let root = tempfile::tempdir().unwrap();

    edit_modprobe_blacklist(root.path(), &module("r8169"), true).unwrap();
    edit_modprobe_blacklist(root.path(), &module("mt76-usb"), true).unwrap();

    assert_eq!(
        edit_modprobe_blacklist(root.path(), &module("mt76_usb"), true).unwrap(),
        FileChange::Unchanged
    );
    assert!(
        fs::read_to_string(file(root.path()))
            .unwrap()
            .ends_with("blacklist r8169\nblacklist mt76_usb\n")
    );
}

#[test]
fn the_last_entry_takes_the_file_with_it() {
    let root = tempfile::tempdir().unwrap();

    edit_modprobe_blacklist(root.path(), &module("wl"), true).unwrap();
    edit_modprobe_blacklist(root.path(), &module("b43"), true).unwrap();

    assert_eq!(
        edit_modprobe_blacklist(root.path(), &module("wl"), false).unwrap(),
        FileChange::Written
    );
    assert!(file(root.path()).exists());
    assert_eq!(
        edit_modprobe_blacklist(root.path(), &module("b43"), false).unwrap(),
        FileChange::Removed
    );
    assert!(!file(root.path()).exists());
}

#[test]
fn removing_what_is_not_listed_writes_and_creates_nothing() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        edit_modprobe_blacklist(root.path(), &module("wl"), false).unwrap(),
        FileChange::Unchanged
    );
    assert!(!root.path().join("etc").exists());
}

/// The package file of the image and an administrator's file are not
/// touched, whatever they hold. Edit: the administrator's file is the
/// package file with `blacklist wl` appended, as the image has no file in
/// `/etc/modprobe.d`.
#[test]
fn other_modprobe_files_stay_as_they_are() {
    let root = tempfile::tempdir().unwrap();
    let etc = root.path().join("etc/modprobe.d");
    let lib = root.path().join("usr/lib/modprobe.d");
    let own = format!("{SYSTEMD_CONF}blacklist wl\n");
    fs::create_dir_all(&etc).unwrap();
    fs::create_dir_all(&lib).unwrap();
    fs::write(etc.join("blacklist.conf"), &own).unwrap();
    fs::write(lib.join("systemd.conf"), SYSTEMD_CONF).unwrap();

    edit_modprobe_blacklist(root.path(), &module("wl"), true).unwrap();
    edit_modprobe_blacklist(root.path(), &module("wl"), false).unwrap();
    edit_modprobe_blacklist(root.path(), &module("bonding"), false).unwrap();

    assert_eq!(fs::read_to_string(etc.join("blacklist.conf")).unwrap(), own);
    assert_eq!(
        fs::read_to_string(lib.join("systemd.conf")).unwrap(),
        SYSTEMD_CONF
    );
}

#[test]
fn a_symlinked_folder_or_file_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    symlink(elsewhere.path(), root.path().join("etc/modprobe.d")).unwrap();

    edit_modprobe_blacklist(root.path(), &module("wl"), true).unwrap_err();
    assert_eq!(fs::read_dir(elsewhere.path()).unwrap().count(), 0);

    let other = tempfile::tempdir().unwrap();
    let target = elsewhere.path().join("target.conf");
    fs::create_dir_all(other.path().join("etc/modprobe.d")).unwrap();
    fs::write(&target, SYSTEMD_CONF).unwrap();
    symlink(&target, file(other.path())).unwrap();

    edit_modprobe_blacklist(other.path(), &module("wl"), true).unwrap_err();
    assert_eq!(fs::read_to_string(&target).unwrap(), SYSTEMD_CONF);
}

#[test]
fn a_file_that_is_no_text_stays_untouched() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc/modprobe.d")).unwrap();
    fs::write(file(root.path()), [0xff_u8, 0xfe_u8]).unwrap();

    edit_modprobe_blacklist(root.path(), &module("wl"), true).unwrap_err();
    assert_eq!(fs::read(file(root.path())).unwrap(), [0xff_u8, 0xfe_u8]);
}
