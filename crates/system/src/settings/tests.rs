use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use tempfile::TempDir;
use toolbox_plan::{DnsProvider, GarudaUpdateFlag, WifiBackend};
use toolbox_state::parts::OwnBlacklist;
use toolbox_state::settings::{CurrentDns, CurrentWifiBackend, GarudaUpdateConfig, HblockState};

use super::{EtcSettings, SettingsSource as _, blocks_domains, parse_dns};
use crate::ProbeError;
use crate::testing::running_as_root;

const HBLOCK_HOSTS: &str = "\
# <custom>
127.0.0.1 localhost.localdomain localhost
# </custom>

# Author:          Héctor Molinero Fernández <hector@molinero.dev>
# Repository:      https://github.com/hectorm/hblock
# Last updated:    Tue, 01 Oct 2026 10:00:00 UTC
# Blocked domains: 284934
0.0.0.0 ads.example
";

fn system() -> (TempDir, EtcSettings) {
    let root = tempfile::tempdir().unwrap();
    let settings = EtcSettings::new(root.path());
    (root, settings)
}

fn conf_d(root: &TempDir) -> PathBuf {
    let folder = root.path().join("etc/NetworkManager/conf.d");
    fs::create_dir_all(&folder).unwrap();
    folder
}

#[test]
fn a_known_server_names_its_provider() {
    assert_eq!(
        parse_dns("[global-dns-domain-*]\nservers=9.9.9.9\n"),
        DnsProvider::Quad9
    );
    assert_eq!(
        parse_dns("[global-dns-domain-*]\nservers = 1.1.1.1, 1.0.0.1\n"),
        DnsProvider::Cloudflare
    );
}

#[test]
fn an_unknown_server_or_none_shows_as_the_default() {
    assert_eq!(parse_dns("servers=192.168.1.1\n"), DnsProvider::Default);
    assert_eq!(parse_dns("[main]\n"), DnsProvider::Default);
}

#[test]
fn dns_comes_from_the_toolbox_drop_in() {
    let (root, settings) = system();

    assert_eq!(settings.dns(), Err(ProbeError::Unavailable));

    let folder = conf_d(&root);
    assert_eq!(settings.dns(), Ok(CurrentDns(DnsProvider::Default)));

    fs::write(
        folder.join("10-garuda-assistant-dns.conf"),
        "[global-dns-domain-*]\nservers=8.8.8.8\n",
    )
    .unwrap();
    assert_eq!(settings.dns(), Ok(CurrentDns(DnsProvider::Google)));
}

#[test]
fn hblock_counts_as_on_while_it_blocks_domains() {
    assert!(blocks_domains(HBLOCK_HOSTS));
    assert!(!blocks_domains(&HBLOCK_HOSTS.replace("284934", "0")));
    assert!(!blocks_domains("127.0.0.1 localhost\n"));
}

#[test]
fn hblock_reads_the_hosts_file() {
    let (root, settings) = system();

    assert_eq!(settings.hblock(), Ok(HblockState(false)));

    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(root.path().join("etc/hosts"), HBLOCK_HOSTS).unwrap();
    assert_eq!(settings.hblock(), Ok(HblockState(true)));
}

#[test]
fn the_wifi_backend_is_read_from_the_drop_ins() {
    let (root, settings) = system();

    assert_eq!(settings.wifi_backend(), Err(ProbeError::Unavailable));

    let folder = conf_d(&root);
    fs::write(
        folder.join("20_wifi_backend_rani.conf"),
        "[device]\nwifi.backend = iwd\n",
    )
    .unwrap();
    assert_eq!(
        settings.wifi_backend(),
        Ok(CurrentWifiBackend(WifiBackend::Iwd))
    );
}

#[test]
fn garuda_update_without_its_folder_is_unavailable() {
    let (root, settings) = system();

    assert_eq!(settings.garuda_update(), Err(ProbeError::Unavailable));

    let folder = root.path().join("etc/garuda/garuda-update");
    fs::create_dir_all(&folder).unwrap();
    assert_eq!(settings.garuda_update(), Ok(GarudaUpdateConfig::default()));

    fs::write(folder.join("config"), "UPDATE_AUR=1\n# NO_CHECK_SPACE=1\n").unwrap();
    assert_eq!(
        settings.garuda_update(),
        Ok(GarudaUpdateConfig(BTreeSet::from([
            GarudaUpdateFlag::UpdateAur
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
    fs::write(folder.join("config"), "UPDATE_AUR=1\n").unwrap();
    fs::set_permissions(folder.join("config"), fs::Permissions::from_mode(0o000)).unwrap();

    assert_eq!(settings.garuda_update(), Err(ProbeError::Unavailable));
}

#[test]
fn the_own_blacklist_file_lists_its_modules() {
    let (root, settings) = system();
    let folder = root.path().join("etc/modprobe.d");
    fs::create_dir_all(&folder).unwrap();
    fs::write(
        folder.join("garuda-toolbox-blacklist.conf"),
        "# Written by Garuda Toolbox\nblacklist mt76-usb\nblacklist wl\n",
    )
    .unwrap();
    fs::write(folder.join("blacklist.conf"), "blacklist b43\n").unwrap();

    let names: Vec<String> = settings
        .module_blacklist()
        .unwrap()
        .0
        .into_iter()
        .map(|module| module.to_string())
        .collect();

    assert_eq!(names, ["mt76_usb", "wl"]);
}

#[test]
fn without_the_own_blacklist_file_nothing_is_blacklisted_by_the_app() {
    let (_root, settings) = system();

    assert_eq!(settings.module_blacklist(), Ok(OwnBlacklist::default()));
}
