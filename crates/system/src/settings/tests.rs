use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use tempfile::TempDir;
use toolbox_plan::{
    DNS_CONFIG, DnsProvider, GarudaUpdateFlag, KernelModule, WIFI_BACKEND_CONFIG, WifiBackend,
    set_blacklisted,
};
use toolbox_state::parts::OwnBlacklist;
use toolbox_state::settings::{CurrentDns, CurrentWifiBackend, GarudaUpdateConfig, HblockState};

use super::{EtcSettings, SettingsSource as _, blocks_domains, parse_dns};
use crate::ProbeError;
use crate::testing::{running_as_root, write};

/// The Lima VM's `/etc/hosts` before and after `hblock`, the latter cut
/// to its first 80 lines.
const HOSTS: &str = include_str!("fixtures/hosts");
const HBLOCK_HOSTS: &str = include_str!("fixtures/hosts-hblock");
/// As garuda-update ships it, every variable commented out.
const GARUDA_UPDATE: &str = include_str!("fixtures/garuda-update-config");
pub(super) const NETWORKMANAGER_CONF: &str =
    include_str!("fixtures/networkmanager/etc/NetworkManager/NetworkManager.conf");
pub(super) const CONNECTIVITY_CONF: &str =
    include_str!("fixtures/networkmanager/usr/lib/NetworkManager/conf.d/20-connectivity.conf");
const SYSTEMD_MODPROBE: &str =
    include_str!("../network/modules/fixtures/root/usr/lib/modprobe.d/systemd.conf");

fn system() -> (TempDir, EtcSettings) {
    let root = tempfile::tempdir().unwrap();
    let settings = EtcSettings::new(root.path());
    (root, settings)
}

/// The VM's NetworkManager files and its empty drop-in folder.
pub(super) fn conf_d(root: &TempDir) -> PathBuf {
    let folder = root.path().join("etc/NetworkManager/conf.d");

    write(
        root.path(),
        "etc/NetworkManager/NetworkManager.conf",
        NETWORKMANAGER_CONF,
    );
    write(
        root.path(),
        "usr/lib/NetworkManager/conf.d/20-connectivity.conf",
        CONNECTIVITY_CONF,
    );
    fs::create_dir_all(&folder).unwrap();
    folder
}

/// The drop-in the helper writes for `provider`.
fn dns_config(provider: DnsProvider) -> String {
    provider.config().unwrap()
}

/// The Cloudflare drop-in edited to blanks around `=` and a second server.
#[test]
fn a_known_server_names_its_provider() {
    let spaced = dns_config(DnsProvider::Cloudflare).replacen(
        "servers=1.1.1.1",
        "servers = 1.1.1.1, 1.0.0.1",
        1,
    );

    assert_eq!(
        parse_dns(&dns_config(DnsProvider::Quad9)),
        DnsProvider::Quad9
    );
    assert_eq!(parse_dns(&spaced), DnsProvider::Cloudflare);
}

/// The Google drop-in edited to the VM's own router, and the VM's main
/// file, which names no server.
#[test]
fn an_unknown_server_or_none_shows_as_the_default() {
    let router = dns_config(DnsProvider::Google).replacen("8.8.8.8", "192.168.5.2", 1);

    assert_eq!(parse_dns(&router), DnsProvider::Default);
    assert_eq!(parse_dns(NETWORKMANAGER_CONF), DnsProvider::Default);
}

#[test]
fn dns_comes_from_the_toolbox_drop_in() {
    let (root, settings) = system();

    assert_eq!(settings.dns(), Err(ProbeError::Unavailable));

    let folder = conf_d(&root);
    assert_eq!(settings.dns(), Ok(CurrentDns(DnsProvider::Default)));

    fs::write(folder.join(DNS_CONFIG), dns_config(DnsProvider::Google)).unwrap();
    assert_eq!(settings.dns(), Ok(CurrentDns(DnsProvider::Google)));
}

/// The hblock file also with its count edited to 0.
#[test]
fn hblock_counts_as_on_while_it_blocks_domains() {
    assert!(blocks_domains(HBLOCK_HOSTS));
    assert!(!blocks_domains(&HBLOCK_HOSTS.replacen(
        "Blocked domains: 478164",
        "Blocked domains: 0",
        1
    )));
    assert!(!blocks_domains(HOSTS));
}

#[test]
fn hblock_reads_the_hosts_file() {
    let (root, settings) = system();

    assert_eq!(settings.hblock(), Ok(HblockState(false)));

    write(root.path(), "etc/hosts", HOSTS);
    assert_eq!(settings.hblock(), Ok(HblockState(false)));

    write(root.path(), "etc/hosts", HBLOCK_HOSTS);
    assert_eq!(settings.hblock(), Ok(HblockState(true)));
}

#[test]
fn the_wifi_backend_is_read_from_the_drop_ins() {
    let (root, settings) = system();

    assert_eq!(settings.wifi_backend(), Err(ProbeError::Unavailable));

    let folder = conf_d(&root);
    assert_eq!(
        settings.wifi_backend(),
        Ok(CurrentWifiBackend(WifiBackend::WpaSupplicant))
    );

    fs::write(
        folder.join(WIFI_BACKEND_CONFIG),
        WifiBackend::Iwd.config().unwrap(),
    )
    .unwrap();
    assert_eq!(
        settings.wifi_backend(),
        Ok(CurrentWifiBackend(WifiBackend::Iwd))
    );
}

/// The shipped config with its `UPDATE_AUR` line uncommented.
#[test]
fn garuda_update_without_its_folder_is_unavailable() {
    let (root, settings) = system();

    assert_eq!(settings.garuda_update(), Err(ProbeError::Unavailable));

    let folder = root.path().join("etc/garuda/garuda-update");
    fs::create_dir_all(&folder).unwrap();
    assert_eq!(settings.garuda_update(), Ok(GarudaUpdateConfig::default()));

    fs::write(folder.join("config"), GARUDA_UPDATE).unwrap();
    assert_eq!(settings.garuda_update(), Ok(GarudaUpdateConfig::default()));

    fs::write(
        folder.join("config"),
        GARUDA_UPDATE.replacen("# UPDATE_AUR=1", "UPDATE_AUR=1", 1),
    )
    .unwrap();
    assert_eq!(
        settings.garuda_update(),
        Ok(GarudaUpdateConfig(BTreeSet::from([
            GarudaUpdateFlag::UpdateAur
        ])))
    );
}

/// Edit: the shipped config with `NO_CHECK_SPACE=1` appended, the name
/// older toolbox versions wrote.
#[test]
fn the_old_space_check_name_reads_as_on() {
    let (root, settings) = system();
    write(
        root.path(),
        "etc/garuda/garuda-update/config",
        format!("{GARUDA_UPDATE}\nNO_CHECK_SPACE=1\n"),
    );

    assert_eq!(
        settings.garuda_update(),
        Ok(GarudaUpdateConfig(BTreeSet::from([
            GarudaUpdateFlag::NoCheckSpace
        ])))
    );
}

#[test]
fn an_unreadable_garuda_update_config_is_unavailable() {
    if running_as_root() {
        return;
    }

    let (root, settings) = system();
    let folder = root.path().join("etc/garuda/garuda-update");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("config"), GARUDA_UPDATE).unwrap();
    fs::set_permissions(folder.join("config"), fs::Permissions::from_mode(0o000)).unwrap();

    assert_eq!(settings.garuda_update(), Err(ProbeError::Unavailable));
}

/// The own file as the helper writes it, and the VM's `systemd.conf` with
/// a blacklist line appended under another name.
#[test]
fn the_own_blacklist_file_lists_its_modules() {
    let (root, settings) = system();
    let module = |name: &str| KernelModule::new(name).unwrap();
    let own = set_blacklisted(
        &set_blacklisted("", &module("mt76-usb"), true),
        &module("wl"),
        true,
    );

    write(
        root.path(),
        "etc/modprobe.d/garuda-toolbox-blacklist.conf",
        own,
    );
    write(
        root.path(),
        "etc/modprobe.d/blacklist.conf",
        format!("{SYSTEMD_MODPROBE}blacklist b43\n"),
    );

    let names: Vec<String> = settings
        .module_blacklist()
        .unwrap()
        .0
        .into_iter()
        .map(|entry| entry.to_string())
        .collect();

    assert_eq!(names, ["mt76_usb", "wl"]);
}

#[test]
fn without_the_own_blacklist_file_nothing_is_blacklisted_by_the_app() {
    let (_root, settings) = system();

    assert_eq!(settings.module_blacklist(), Ok(OwnBlacklist::default()));
}
