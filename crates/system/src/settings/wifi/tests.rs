use std::os::unix::fs::symlink;

use tempfile::TempDir;
use toolbox_plan::WifiBackend;

use super::{backend, backend_of};
use crate::ProbeError;
use crate::settings::tests::{CONNECTIVITY_CONF, NETWORKMANAGER_CONF, conf_d};
use crate::test_support::write;

/// The drop-in the helper writes for iwd.
fn iwd() -> &'static str {
    WifiBackend::Iwd.config().unwrap()
}

/// The helper writes no file for `wpa_supplicant`, so this is its iwd
/// drop-in with the value replaced.
fn supplicant() -> String {
    iwd().replacen("=iwd", "=wpa_supplicant", 1)
}

/// The VM's NetworkManager files plus `files`.
fn system(files: &[(&str, &str)]) -> TempDir {
    let root = tempfile::tempdir().unwrap();

    conf_d(&root);

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
}

#[test]
fn every_folder_and_the_main_file_can_select_iwd() {
    for path in [
        "usr/lib/NetworkManager/conf.d/wifi.conf",
        "run/NetworkManager/conf.d/wifi.conf",
        "etc/NetworkManager/NetworkManager.conf",
        "etc/NetworkManager/conf.d/wifi.conf",
    ] {
        assert_eq!(chosen(&[(path, iwd())]), WifiBackend::Iwd, "{path}");
    }
}

#[test]
fn the_file_read_later_overrides_the_earlier() {
    let supplicant = supplicant();
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
            chosen(&[(earlier, iwd()), (later, &supplicant)]),
            WifiBackend::WpaSupplicant,
            "{later} after {earlier}"
        );
        assert_eq!(
            chosen(&[(earlier, &supplicant), (later, iwd())]),
            WifiBackend::Iwd,
            "{later} after {earlier}"
        );
    }
}

/// The VM's `20-connectivity.conf` selects nothing, so masking an iwd
/// drop-in with it leaves the default.
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
            chosen(&[(masked, iwd()), (mask, CONNECTIVITY_CONF)]),
            WifiBackend::WpaSupplicant,
            "{mask} over {masked}"
        );
    }

    let root = system(&[("usr/lib/NetworkManager/conf.d/wifi.conf", iwd())]);

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
        chosen(&[("etc/NetworkManager/conf.d/iwd.txt", iwd())]),
        WifiBackend::WpaSupplicant
    );
}

/// The VM's files and the helper's iwd drop-in, edited: the key commented
/// out, the section renamed or dropped, a device match added, blanks
/// around the value, another section after it, the key repeated.
#[test]
fn the_last_key_of_the_device_section_is_the_answer() {
    let iwd = iwd();
    let supplicant = supplicant();
    let cases = [
        (String::new(), None),
        (NETWORKMANAGER_CONF.to_owned(), None),
        (CONNECTIVITY_CONF.to_owned(), None),
        (iwd.replacen("wifi.backend", "#wifi.backend", 1), None),
        (iwd.replacen("[device]", "[main]", 1), None),
        (iwd.replacen("[device]\n", "", 1), None),
        (
            iwd.replacen(
                "[device]",
                "[device-wlan0]\nmatch-device=interface-name:wlan0",
                1,
            ),
            None,
        ),
        (iwd.replacen("=iwd", " = iwd ", 1), Some("iwd")),
        (
            format!(
                "{}{CONNECTIVITY_CONF}",
                iwd.replacen("[device]", " [device] ", 1)
            ),
            Some("iwd"),
        ),
        (
            format!("{iwd}{}", supplicant.replacen("[device]\n", "", 1)),
            Some("wpa_supplicant"),
        ),
        (format!("{supplicant}{CONNECTIVITY_CONF}{iwd}"), Some("iwd")),
    ];

    for (text, expected) in cases {
        assert_eq!(backend_of(&text), expected, "{text}");
    }
}
