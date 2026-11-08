use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use toolbox_plan::{DnsProvider, GarudaUpdateFlag, ShellPath, WifiBackend};

use super::{
    FileChange, set_garuda_update_flag, shell_is_listed, write_dns_config, write_wifi_backend,
};
use crate::testing::mode;

fn system_with_network_manager() -> (TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let conf_d = root.path().join("etc/NetworkManager/conf.d");
    fs::create_dir_all(&conf_d).unwrap();
    (root, conf_d)
}

fn shell(path: &str) -> ShellPath {
    ShellPath::new(path).unwrap()
}

#[test]
fn only_exactly_listed_shells_pass() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(
        root.path().join("etc/shells"),
        "# Pathnames of valid login shells.\n/bin/sh\n/usr/bin/bash\n  /usr/bin/fish  \n# /usr/bin/zsh\n",
    )
    .unwrap();

    assert!(shell_is_listed(root.path(), &shell("/usr/bin/bash")).unwrap());
    assert!(shell_is_listed(root.path(), &shell("/usr/bin/fish")).unwrap());
    assert!(!shell_is_listed(root.path(), &shell("/usr/bin/zsh")).unwrap());
    assert!(!shell_is_listed(root.path(), &shell("/usr/bin/false")).unwrap());
    assert!(!shell_is_listed(root.path(), &shell("/usr/bin/bas")).unwrap());
}

#[test]
fn a_missing_shell_list_is_an_error_not_a_pass() {
    let root = tempfile::tempdir().unwrap();

    shell_is_listed(root.path(), &shell("/usr/bin/bash")).unwrap_err();
}

#[test]
fn a_dns_provider_writes_the_drop_in_world_readable() {
    let (root, conf_d) = system_with_network_manager();

    assert_eq!(
        write_dns_config(root.path(), DnsProvider::Google).unwrap(),
        FileChange::Written
    );

    let file = conf_d.join("10-garuda-assistant-dns.conf");
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "[global-dns-domain-*]\nservers=8.8.8.8\n"
    );
    assert_eq!(mode(&file), 0o644);
}

#[test]
fn the_default_dns_removes_the_drop_in_once() {
    let (root, conf_d) = system_with_network_manager();
    fs::write(conf_d.join("10-garuda-assistant-dns.conf"), "old").unwrap();

    assert_eq!(
        write_dns_config(root.path(), DnsProvider::Default).unwrap(),
        FileChange::Removed
    );
    assert!(!conf_d.join("10-garuda-assistant-dns.conf").exists());
    assert_eq!(
        write_dns_config(root.path(), DnsProvider::Default).unwrap(),
        FileChange::Unchanged
    );
}

#[test]
fn dns_needs_network_manager_installed() {
    let root = tempfile::tempdir().unwrap();

    write_dns_config(root.path(), DnsProvider::Quad9).unwrap_err();
    assert!(!root.path().join("etc").exists());
}

#[test]
fn a_symlinked_drop_in_folder_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc/NetworkManager")).unwrap();
    std::os::unix::fs::symlink(
        elsewhere.path(),
        root.path().join("etc/NetworkManager/conf.d"),
    )
    .unwrap();

    write_dns_config(root.path(), DnsProvider::Quad9).unwrap_err();
    assert_eq!(fs::read_dir(elsewhere.path()).unwrap().count(), 0);
}

#[test]
fn iwd_writes_the_backend_drop_in() {
    let (root, conf_d) = system_with_network_manager();

    assert_eq!(
        write_wifi_backend(root.path(), WifiBackend::Iwd).unwrap(),
        FileChange::Written
    );
    assert_eq!(
        fs::read_to_string(conf_d.join("20_wifi_backend_toolbox.conf")).unwrap(),
        "[device]\nwifi.backend=iwd\n"
    );
}

#[test]
fn wpa_supplicant_removes_both_backend_drop_ins() {
    let (root, conf_d) = system_with_network_manager();
    fs::write(conf_d.join("20_wifi_backend_rani.conf"), "x").unwrap();
    fs::write(conf_d.join("other.conf"), "keep").unwrap();

    assert_eq!(
        write_wifi_backend(root.path(), WifiBackend::WpaSupplicant).unwrap(),
        FileChange::Removed
    );
    assert!(!conf_d.join("20_wifi_backend_rani.conf").exists());
    assert!(conf_d.join("other.conf").exists());
    assert_eq!(
        write_wifi_backend(root.path(), WifiBackend::WpaSupplicant).unwrap(),
        FileChange::Unchanged
    );
}

#[test]
fn a_garuda_update_switch_creates_folder_and_file() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        set_garuda_update_flag(root.path(), GarudaUpdateFlag::UpdateAur, true).unwrap(),
        FileChange::Written
    );

    let config = root.path().join("etc/garuda/garuda-update/config");
    assert_eq!(fs::read_to_string(&config).unwrap(), "UPDATE_AUR=1\n");
    assert_eq!(mode(&config), 0o644);
}

#[test]
fn a_garuda_update_switch_keeps_the_rest_of_the_file() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("etc/garuda/garuda-update");
    fs::create_dir_all(&folder).unwrap();
    fs::write(
        folder.join("config"),
        "# comment\nSKIP_MIRRORLIST=1\nMY_OWN=\"x y\"\n",
    )
    .unwrap();

    assert_eq!(
        set_garuda_update_flag(root.path(), GarudaUpdateFlag::SkipMirrorlist, false).unwrap(),
        FileChange::Written
    );
    assert_eq!(
        fs::read_to_string(folder.join("config")).unwrap(),
        "# comment\n# SKIP_MIRRORLIST=1\nMY_OWN=\"x y\"\n"
    );
    assert_eq!(
        set_garuda_update_flag(root.path(), GarudaUpdateFlag::SkipMirrorlist, false).unwrap(),
        FileChange::Unchanged
    );
}

#[test]
fn switching_off_without_a_file_creates_nothing() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        set_garuda_update_flag(root.path(), GarudaUpdateFlag::NoCheckSpace, false).unwrap(),
        FileChange::Unchanged
    );
    assert!(!root.path().join("etc/garuda/garuda-update/config").exists());
}

#[test]
fn a_config_that_is_not_text_is_left_alone() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("etc/garuda/garuda-update");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("config"), [0xff_u8, 0xfe_u8]).unwrap();

    set_garuda_update_flag(root.path(), GarudaUpdateFlag::UpdateAur, true).unwrap_err();
    assert_eq!(fs::read(folder.join("config")).unwrap(), [0xff_u8, 0xfe_u8]);
}
