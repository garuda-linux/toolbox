use std::fs;
use std::path::Path;

use toolbox_plan::sdboot::{LoaderChange, LoaderSetting, LoaderTimeout};

use super::edit_loader_conf;
use crate::builtin::FileChange;

/// Files of the Garuda Lima image.
const MOUNTINFO: &[u8] = include_bytes!("../../../../plan/src/sdboot/fixtures/mountinfo");
const LOADER_CONF: &str = include_str!("../../../../plan/src/sdboot/fixtures/loader.conf");

fn image(root: &Path) {
    fs::create_dir_all(root.join("proc/self")).unwrap();
    fs::write(root.join("proc/self/mountinfo"), MOUNTINFO).unwrap();
    fs::create_dir_all(root.join("efi/loader")).unwrap();
    fs::write(root.join("efi/loader/loader.conf"), LOADER_CONF).unwrap();
}

fn timeout(seconds: u32) -> LoaderChange {
    LoaderChange::new(vec![LoaderSetting::Timeout(Some(LoaderTimeout::Seconds(
        seconds,
    )))])
    .unwrap()
}

#[test]
fn the_images_file_gets_the_setting_and_keeps_the_old_one_beside_it() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());

    let change = edit_loader_conf(dir.path(), &timeout(5)).unwrap();

    assert_eq!(change, FileChange::Written);
    assert_eq!(
        fs::read_to_string(dir.path().join("efi/loader/loader.conf")).unwrap(),
        "timeout 5\n#console-mode keep\n"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("efi/loader/loader.conf.bak")).unwrap(),
        LOADER_CONF
    );
}

/// systemd-boot runs on its defaults without the file, so the first
/// setting makes one.
#[test]
fn an_esp_without_loader_conf_gets_one() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());
    fs::remove_file(dir.path().join("efi/loader/loader.conf")).unwrap();

    edit_loader_conf(dir.path(), &timeout(5)).unwrap();

    assert_eq!(
        fs::read_to_string(dir.path().join("efi/loader/loader.conf")).unwrap(),
        "timeout 5\n"
    );
}

#[test]
fn a_setting_the_file_already_has_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());
    edit_loader_conf(dir.path(), &timeout(5)).unwrap();

    assert_eq!(
        edit_loader_conf(dir.path(), &timeout(5)).unwrap(),
        FileChange::Unchanged
    );
}

/// The image's mount table without its ESP line: nothing is written
/// anywhere.
#[test]
fn without_a_mounted_esp_nothing_is_written() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());

    let without = String::from_utf8(MOUNTINFO.to_vec())
        .unwrap()
        .lines()
        .filter(|line| !line.contains(" /efi "))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(dir.path().join("proc/self/mountinfo"), without).unwrap();

    edit_loader_conf(dir.path(), &timeout(5)).unwrap_err();
    assert_eq!(
        fs::read_to_string(dir.path().join("efi/loader/loader.conf")).unwrap(),
        LOADER_CONF
    );
}
