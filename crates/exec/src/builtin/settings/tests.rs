use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use toolbox_plan::{DnsProvider, GarudaUpdateFlag, ShellPath, WifiBackend};

use super::{
    FileChange, set_garuda_update_flag, shell_is_listed, write_dns_config, write_wifi_backend,
};
use crate::testing::mode;

/// `/etc/NetworkManager/NetworkManager.conf` of the Lima Garuda image,
/// whose `conf.d` beside it is empty.
const NETWORKMANAGER_CONF: &str = include_str!(
    "../../../../system/src/settings/fixtures/networkmanager/etc/NetworkManager/NetworkManager.conf"
);

/// `/etc/shells` of the Lima Garuda image.
const SHELLS: &str = include_str!("../../../../system/src/accounts/fixtures/shells");

/// `/etc/garuda/garuda-update/config` as garuda-update ships it, without
/// a line break at the end.
const GARUDA_UPDATE: &str =
    include_str!("../../../../system/src/settings/fixtures/garuda-update-config");

fn system_with_network_manager() -> (TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let conf_d = root.path().join("etc/NetworkManager/conf.d");
    fs::create_dir_all(&conf_d).unwrap();
    fs::write(
        root.path().join("etc/NetworkManager/NetworkManager.conf"),
        NETWORKMANAGER_CONF,
    )
    .unwrap();
    (root, conf_d)
}

fn main_config_is_untouched(root: &TempDir) {
    assert_eq!(
        fs::read_to_string(root.path().join("etc/NetworkManager/NetworkManager.conf")).unwrap(),
        NETWORKMANAGER_CONF
    );
}

fn shell(path: &str) -> ShellPath {
    ShellPath::new(path).unwrap()
}

fn root_with_shells(shells: &str) -> TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/shells"), shells).unwrap();
    root
}

#[test]
fn only_exactly_listed_shells_pass() {
    let root = root_with_shells(SHELLS);

    assert!(shell_is_listed(root.path(), &shell("/usr/bin/bash")).unwrap());
    assert!(shell_is_listed(root.path(), &shell("/bin/sh")).unwrap());
    assert!(!shell_is_listed(root.path(), &shell("/usr/bin/fish")).unwrap());
    assert!(!shell_is_listed(root.path(), &shell("/usr/bin/false")).unwrap());
    assert!(!shell_is_listed(root.path(), &shell("/usr/bin/bas")).unwrap());
}

/// Edits: `  /usr/bin/fish  ` with blanks around it and `# /usr/bin/zsh`
/// appended, as a hand-edited file could have them.
#[test]
fn blanks_around_a_shell_count_and_a_comment_does_not() {
    let root = root_with_shells(&format!("{SHELLS}  /usr/bin/fish  \n# /usr/bin/zsh\n"));

    assert!(shell_is_listed(root.path(), &shell("/usr/bin/fish")).unwrap());
    assert!(!shell_is_listed(root.path(), &shell("/usr/bin/zsh")).unwrap());
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
    main_config_is_untouched(&root);
}

#[test]
fn the_default_dns_removes_the_drop_in_once() {
    let (root, conf_d) = system_with_network_manager();
    write_dns_config(root.path(), DnsProvider::Quad9).unwrap();

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

/// The older tool's drop-in is written here with the content the toolbox
/// writes for iwd, because no capture of it exists. A package drop-in of
/// the image is copied into `conf.d` beside it to show that it stays.
#[test]
fn wpa_supplicant_removes_both_backend_drop_ins() {
    let (root, conf_d) = system_with_network_manager();
    let connectivity = include_str!(
        "../../../../system/src/settings/fixtures/networkmanager/usr/lib/NetworkManager/conf.d/20-connectivity.conf"
    );

    fs::write(
        conf_d.join("20_wifi_backend_rani.conf"),
        WifiBackend::Iwd.config().unwrap(),
    )
    .unwrap();
    fs::write(conf_d.join("20-connectivity.conf"), connectivity).unwrap();

    assert_eq!(
        write_wifi_backend(root.path(), WifiBackend::WpaSupplicant).unwrap(),
        FileChange::Removed
    );
    assert!(!conf_d.join("20_wifi_backend_rani.conf").exists());
    assert_eq!(
        fs::read_to_string(conf_d.join("20-connectivity.conf")).unwrap(),
        connectivity
    );
    main_config_is_untouched(&root);
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

fn root_with_garuda_update(config: impl AsRef<[u8]>) -> (TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("etc/garuda/garuda-update");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("config"), config).unwrap();
    (root, folder.join("config"))
}

#[test]
fn a_garuda_update_switch_revives_its_shipped_line_and_back() {
    let (root, config) = root_with_garuda_update(GARUDA_UPDATE);

    assert_eq!(
        set_garuda_update_flag(root.path(), GarudaUpdateFlag::SkipMirrorlist, true).unwrap(),
        FileChange::Written
    );
    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        GARUDA_UPDATE.replace("# SKIP_MIRRORLIST=1", "SKIP_MIRRORLIST=1")
    );
    assert_eq!(
        set_garuda_update_flag(root.path(), GarudaUpdateFlag::SkipMirrorlist, false).unwrap(),
        FileChange::Written
    );
    assert_eq!(fs::read_to_string(&config).unwrap(), GARUDA_UPDATE);
    assert_eq!(
        set_garuda_update_flag(root.path(), GarudaUpdateFlag::SkipMirrorlist, false).unwrap(),
        FileChange::Unchanged
    );
}

/// Edit: `MY_OWN="x y"` appended after a line break, a setting of the
/// administrator the switch must keep.
#[test]
fn a_garuda_update_switch_keeps_the_rest_of_the_file() {
    let own = format!("{GARUDA_UPDATE}\nMY_OWN=\"x y\"\n");
    let (root, config) = root_with_garuda_update(&own);

    set_garuda_update_flag(root.path(), GarudaUpdateFlag::UpdateAur, true).unwrap();

    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        own.replace("# UPDATE_AUR=1", "UPDATE_AUR=1")
    );
}

#[test]
fn the_space_check_revives_the_shipped_line() {
    let (root, config) = root_with_garuda_update(GARUDA_UPDATE);

    set_garuda_update_flag(root.path(), GarudaUpdateFlag::NoCheckSpace, true).unwrap();

    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        GARUDA_UPDATE.replace("# NO_SPACE_CHECK=1", "NO_SPACE_CHECK=1")
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
    let (root, config) = root_with_garuda_update([0xff_u8, 0xfe_u8]);

    set_garuda_update_flag(root.path(), GarudaUpdateFlag::UpdateAur, true).unwrap_err();
    assert_eq!(fs::read(config).unwrap(), [0xff_u8, 0xfe_u8]);
}
