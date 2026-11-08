use std::fs;
use std::os::unix::fs::symlink;

use tempfile::TempDir;
use toolbox_plan::WifiBackend;

use super::{backend, backend_of};
use crate::ProbeError;
use crate::testing::write;

const IWD: &str = "[device]\nwifi.backend=iwd\n";
const SUPPLICANT: &str = "[device]\nwifi.backend=wpa_supplicant\n";

fn system(files: &[(&str, &str)]) -> TempDir {
    let root = tempfile::tempdir().unwrap();

    fs::create_dir_all(root.path().join("etc/NetworkManager/conf.d")).unwrap();

    for (path, text) in files {
        write(root.path(), path, text);
    }

    root
}

fn chosen(files: &[(&str, &str)]) -> WifiBackend {
    backend(system(files).path()).unwrap()
}

#[test]
fn without_the_drop_in_folder_networkmanager_is_not_there() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(backend(root.path()), Err(ProbeError::Unavailable));
}

#[test]
fn nothing_said_means_wpa_supplicant() {
    assert_eq!(chosen(&[]), WifiBackend::WpaSupplicant);
    assert_eq!(
        chosen(&[(
            "etc/NetworkManager/conf.d/dns.conf",
            "[main]\ndns=default\n"
        )]),
        WifiBackend::WpaSupplicant
    );
}

#[test]
fn every_folder_and_the_main_file_can_select_iwd() {
    for path in [
        "usr/lib/NetworkManager/conf.d/wifi.conf",
        "run/NetworkManager/conf.d/wifi.conf",
        "etc/NetworkManager/NetworkManager.conf",
        "etc/NetworkManager/conf.d/wifi.conf",
    ] {
        assert_eq!(chosen(&[(path, IWD)]), WifiBackend::Iwd, "{path}");
    }
}

#[test]
fn the_file_read_later_overrides_the_earlier() {
    let order = [
        "usr/lib/NetworkManager/conf.d/10-a.conf",
        "usr/lib/NetworkManager/conf.d/90-b.conf",
        "run/NetworkManager/conf.d/00-c.conf",
        "etc/NetworkManager/NetworkManager.conf",
        "etc/NetworkManager/conf.d/00-d.conf",
        "etc/NetworkManager/conf.d/50-e.conf",
    ];

    for (earlier, later) in order.iter().zip(order.iter().skip(1)) {
        assert_eq!(
            chosen(&[(earlier, IWD), (later, SUPPLICANT)]),
            WifiBackend::WpaSupplicant,
            "{later} after {earlier}"
        );
        assert_eq!(
            chosen(&[(earlier, SUPPLICANT), (later, IWD)]),
            WifiBackend::Iwd,
            "{later} after {earlier}"
        );
    }
}

#[test]
fn a_file_of_the_same_name_masks_the_one_in_a_lower_folder() {
    for (masked, mask) in [
        (
            "usr/lib/NetworkManager/conf.d/wifi.conf",
            "run/NetworkManager/conf.d/wifi.conf",
        ),
        (
            "usr/lib/NetworkManager/conf.d/wifi.conf",
            "etc/NetworkManager/conf.d/wifi.conf",
        ),
        (
            "run/NetworkManager/conf.d/wifi.conf",
            "etc/NetworkManager/conf.d/wifi.conf",
        ),
    ] {
        assert_eq!(
            chosen(&[(masked, IWD), (mask, "")]),
            WifiBackend::WpaSupplicant,
            "{mask} over {masked}"
        );
    }

    let root = system(&[("usr/lib/NetworkManager/conf.d/wifi.conf", IWD)]);

    symlink(
        "/dev/null",
        root.path().join("etc/NetworkManager/conf.d/wifi.conf"),
    )
    .unwrap();
    assert_eq!(backend(root.path()), Ok(WifiBackend::WpaSupplicant));
}

#[test]
fn only_conf_files_count() {
    assert_eq!(
        chosen(&[("etc/NetworkManager/conf.d/iwd.txt", IWD)]),
        WifiBackend::WpaSupplicant
    );
}

#[test]
fn the_last_key_of_the_device_section_is_the_answer() {
    for (text, expected) in [
        ("", None),
        ("[device]\n#wifi.backend=iwd\n", None),
        ("[main]\nwifi.backend=iwd\n", None),
        ("wifi.backend=iwd\n", None),
        (
            "[device-wlan0]\nmatch-device=interface-name:wlan0\nwifi.backend=iwd\n",
            None,
        ),
        ("[device]\nwifi.backend = iwd \n", Some("iwd")),
        (
            " [device] \nwifi.backend=iwd\n[main]\ndns=default\n",
            Some("iwd"),
        ),
        (
            "[device]\nwifi.backend=iwd\nwifi.backend=wpa_supplicant\n",
            Some("wpa_supplicant"),
        ),
        (
            "[device]\nwifi.backend=wpa_supplicant\n[main]\n[device]\nwifi.backend=iwd\n",
            Some("iwd"),
        ),
    ] {
        assert_eq!(backend_of(text), expected, "{text}");
    }
}
