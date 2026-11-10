use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use toolbox_plan::KernelRelease;
use toolbox_plan::sdboot::{LoaderSettings, LoaderTimeout, MAX_ENTRIES};

use super::sdboot_menu as menu;

/// Files of the Garuda Lima image. A UKI is cut after `.uname`, the last
/// part the read takes.
const MOUNTINFO: &[u8] = toolbox_plan::testing::sdboot::MOUNTINFO.as_bytes();
const LOADER_CONF: &[u8] = toolbox_plan::testing::sdboot::LOADER_CONF.as_bytes();
const UKI: &[u8] = toolbox_plan::testing::sdboot::GARUDA_7_3_1_1_GARUDA_EFI_HEAD;
const FALLBACK: &[u8] = toolbox_plan::testing::sdboot::GARUDA_7_3_1_1_GARUDA_FALLBACK_EFI_HEAD;

fn write(root: &Path, path: &str, content: &[u8]) {
    let path = root.join(path);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn image(root: &Path) {
    write(root, "proc/self/mountinfo", MOUNTINFO);
    write(root, "efi/loader/loader.conf", LOADER_CONF);
    write(root, "efi/EFI/Linux/garuda-7.3.1-1-garuda.efi", UKI);
    write(
        root,
        "efi/EFI/Linux/garuda-7.3.1-1-garuda-fallback.efi",
        FALLBACK,
    );
    fs::create_dir_all(root.join("efi/loader/entries")).unwrap();
}

#[test]
fn the_images_esp_gives_its_two_ukis_and_default_settings() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());

    let menu = menu(dir.path()).unwrap();
    let shown: Vec<(&str, &str)> = menu
        .entries
        .iter()
        .map(|entry| (entry.id.as_str(), entry.title.as_str()))
        .collect();

    assert_eq!(menu.settings, LoaderSettings::default());
    assert_eq!(
        shown,
        [
            (
                "garuda-7.3.1-1-garuda-fallback.efi",
                "Garuda Linux (fallback initramfs)"
            ),
            ("garuda-7.3.1-1-garuda.efi", "Garuda Linux"),
        ]
    );
}

/// The image's UKI with a boot counter, as before its first boot.
#[test]
fn a_counted_uki_has_the_id_without_the_counter() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "proc/self/mountinfo", MOUNTINFO);
    write(dir.path(), "efi/EFI/Linux/garuda-7.3.1-1-garuda+3.efi", UKI);

    let menu = menu(dir.path()).unwrap();

    assert_eq!(menu.entries[0].id.as_str(), "garuda-7.3.1-1-garuda.efi");
    assert_eq!(
        menu.entries[0].uname.as_ref().map(KernelRelease::as_str),
        Some("7.3.1-1-garuda")
    );
}

/// The image without `loader.conf`: systemd-boot runs on its defaults.
#[test]
fn an_esp_without_loader_conf_has_default_settings() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());
    fs::remove_file(dir.path().join("efi/loader/loader.conf")).unwrap();

    assert_eq!(
        menu(dir.path()).unwrap().settings,
        LoaderSettings::default()
    );
}

/// The image's `loader.conf` with its commented timeout switched on.
#[test]
fn the_settings_come_from_the_esps_loader_conf() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());
    let conf = String::from_utf8(LOADER_CONF.to_vec())
        .unwrap()
        .replace("#timeout", "timeout");

    write(dir.path(), "efi/loader/loader.conf", conf.as_bytes());

    assert_eq!(
        menu(dir.path()).unwrap().settings.timeout,
        Some(LoaderTimeout::Seconds(3))
    );
}

/// The image's UKI with its header cut off: it still boots, so it keeps
/// its place.
#[test]
fn a_uki_whose_header_cannot_be_read_keeps_its_name_as_title() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());
    write(
        dir.path(),
        "efi/EFI/Linux/garuda-7.3.1-1-garuda.efi",
        &UKI[0x200..],
    );

    let menu = menu(dir.path()).unwrap();
    let cut = menu
        .entries
        .iter()
        .find(|entry| entry.id.as_str() == "garuda-7.3.1-1-garuda.efi")
        .unwrap();

    assert_eq!(cut.title, "garuda-7.3.1-1-garuda.efi");
    assert_eq!(cut.uname, None);
}

/// The image's mount table without its ESP line.
#[test]
fn without_a_mounted_esp_there_is_no_menu() {
    let dir = tempfile::tempdir().unwrap();
    let without = String::from_utf8(MOUNTINFO.to_vec())
        .unwrap()
        .lines()
        .filter(|line| !line.contains(" /efi "))
        .collect::<Vec<_>>()
        .join("\n");

    write(dir.path(), "proc/self/mountinfo", without.as_bytes());

    menu(dir.path()).unwrap_err();
}

/// The image's mount table with the lines before its ESP repeated 30
/// times, as a machine full of container mounts has it.
#[test]
fn an_esp_far_down_a_long_mount_table_is_found() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());
    let table = String::from_utf8(MOUNTINFO.to_vec()).unwrap();
    let (before, from_esp) = table.split_at(table.find("58 43 254:1 / /efi ").unwrap());
    let long = format!("{}{from_esp}", before.repeat(30));

    assert!(long.len() > 64_usize << 10_u32);
    write(dir.path(), "proc/self/mountinfo", long.as_bytes());

    assert_eq!(menu(dir.path()).unwrap().entries.len(), 2);
}

/// The image's UKI also lies in the entries folder as a `.conf`,
/// which is no text, and its fallback UKI is made unreadable. The menu
/// keeps the main UKI and counts the two it could not read.
#[test]
fn entries_that_cannot_be_read_are_counted() {
    if rustix::process::geteuid().is_root() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    image(dir.path());
    write(dir.path(), "efi/loader/entries/binary.conf", UKI);

    let fallback = dir
        .path()
        .join("efi/EFI/Linux/garuda-7.3.1-1-garuda-fallback.efi");

    fs::set_permissions(&fallback, fs::Permissions::from_mode(0o000)).unwrap();

    let menu = menu(dir.path()).unwrap();

    assert_eq!(menu.entries.len(), 1);
    assert_eq!(menu.skipped, 2);
    assert_eq!(menu.cut, 0);
}

/// The image's UKI is copied under more names than the menu
/// holds, each copy cut to the header's second 512 bytes, so each counts
/// as an entry titled by its name.
#[test]
fn entries_past_the_limit_are_cut_and_counted() {
    let dir = tempfile::tempdir().unwrap();
    image(dir.path());

    for number in 0..MAX_ENTRIES {
        write(
            dir.path(),
            &format!("efi/EFI/Linux/copy-{number:03}.efi"),
            &UKI[0x200..0x400],
        );
    }

    let menu = menu(dir.path()).unwrap();

    assert_eq!(menu.entries.len(), MAX_ENTRIES);
    assert_eq!(menu.cut, 2);
    assert_eq!(menu.skipped, 0);
}
