mod blacklist;
mod drivers;
mod hardware;
mod switching;

use std::net::IpAddr;

use toolbox_plan::{HopLimit, HostName, PingCount, PingTimeout, PlannedAction, UnitName};

use super::{
    Connectivity, GEOCLUE, HOTSPOT_PACKAGE, PackageState, Radio, RadioKind, RadioSummary,
    RadioTarget, flight_mode, package_state, plan_ping, plan_start, plan_traceroute,
    public_address, radio_summary, rfkill_arguments, service_rows, toggle_install,
};
use crate::domains::InstalledPackages;
use crate::features::services::{ActiveState, FileState, UnitInfo};
use crate::test_support::{commands, packages};
use crate::{ChangeSet, Snapshot};

fn radio(kind: Option<RadioKind>, soft: bool, hard: bool) -> Radio {
    Radio { kind, soft, hard }
}

fn unit(name: &str, active: ActiveState, file_state: FileState) -> UnitInfo {
    UnitInfo {
        name: UnitName::new(name).unwrap(),
        description: String::new(),
        active,
        sub_state: String::new(),
        file_state,
    }
}

fn installed(names: &[&str]) -> Snapshot {
    let mut snapshot = Snapshot::default();

    snapshot.insert(InstalledPackages(packages(names)));
    snapshot
}

#[test]
fn connectivity_follows_network_managers_codes() {
    let states: Vec<Connectivity> = (0_u32..=5).map(Connectivity::from_code).collect();

    assert_eq!(
        states,
        [
            Connectivity::Unknown,
            Connectivity::None,
            Connectivity::Portal,
            Connectivity::Limited,
            Connectivity::Full,
            Connectivity::Unknown
        ]
    );
}

#[test]
fn a_kind_is_blocked_when_each_of_its_radios_is() {
    let radios = [
        radio(Some(RadioKind::Wifi), true, false),
        radio(Some(RadioKind::Wifi), false, false),
        radio(Some(RadioKind::Bluetooth), false, true),
    ];

    assert_eq!(
        radio_summary(&radios, RadioKind::Wifi),
        RadioSummary {
            present: true,
            blocked: false,
            hard: false
        }
    );
    assert_eq!(
        radio_summary(&radios, RadioKind::Bluetooth),
        RadioSummary {
            present: true,
            blocked: true,
            hard: true
        }
    );
    assert_eq!(
        radio_summary(&radios[..2], RadioKind::Bluetooth),
        RadioSummary::default()
    );
}

#[test]
fn flight_mode_needs_every_shown_radio_blocked() {
    let wifi_off = radio(Some(RadioKind::Wifi), true, false);
    let bluetooth_on = radio(Some(RadioKind::Bluetooth), false, false);
    let bluetooth_off = radio(Some(RadioKind::Bluetooth), true, false);
    let nfc_on = radio(None, false, false);

    assert!(!flight_mode(&[]));
    assert!(!flight_mode(std::slice::from_ref(&nfc_on)));
    assert!(!flight_mode(&[wifi_off.clone(), bluetooth_on]));
    assert!(flight_mode(&[wifi_off, bluetooth_off, nfc_on]));
}

#[test]
fn the_kernel_names_wifi_wlan() {
    assert_eq!(RadioKind::from_sysfs("wlan\n"), Some(RadioKind::Wifi));
    assert_eq!(
        RadioKind::from_sysfs("bluetooth"),
        Some(RadioKind::Bluetooth)
    );
    assert_eq!(RadioKind::from_sysfs("nfc"), None);
}

#[test]
fn rfkill_gets_two_fixed_words() {
    assert_eq!(rfkill_arguments(RadioTarget::Wifi, true), ["block", "wifi"]);
    assert_eq!(
        rfkill_arguments(RadioTarget::Bluetooth, false),
        ["unblock", "bluetooth"]
    );
    assert_eq!(rfkill_arguments(RadioTarget::All, true), ["block", "all"]);
}

#[test]
fn services_show_as_systemd_lists_them() {
    let units = [
        unit(
            "NetworkManager.service",
            ActiveState::Active,
            FileState::Enabled,
        ),
        unit(
            "bluetooth.service",
            ActiveState::Inactive,
            FileState::Disabled,
        ),
        unit(GEOCLUE, ActiveState::Inactive, FileState::Static),
        unit("sshd.service", ActiveState::Active, FileState::Enabled),
    ];

    let rows: Vec<(&str, bool, &str, &str)> = service_rows(&units)
        .into_iter()
        .map(|row| {
            (
                row.unit,
                row.installed,
                row.active.as_str(),
                row.file_state.as_str(),
            )
        })
        .collect();

    assert_eq!(
        rows,
        [
            ("NetworkManager.service", true, "active", "enabled"),
            ("connman.service", false, "inactive", "unknown"),
            ("bluetooth.service", true, "inactive", "disabled"),
            ("geoclue.service", true, "inactive", "static"),
        ]
    );
}

#[test]
fn only_the_listed_services_start_from_this_page() {
    assert_eq!(
        commands(&plan_start("bluetooth.service").unwrap()),
        ["systemctl start -- bluetooth.service"]
    );
    assert!(plan_start("sshd.service").is_none());
    assert!(plan_start(GEOCLUE).is_none());
    assert!(plan_start("bluetooth.service --now").is_none());
}

#[test]
fn only_the_ip_element_of_the_answer_is_read() {
    let answer = "<Response>\n  <Ip>203.0.113.7</Ip>\n  <Status>OK</Status>\n  \
                  <CountryCode>DE</CountryCode>\n  <City>Berlin</City>\n</Response>\n";

    assert_eq!(
        public_address(answer),
        Some("203.0.113.7".parse::<IpAddr>().unwrap())
    );
    assert_eq!(
        public_address("<Response><Ip> 2001:db8::7 </Ip></Response>"),
        Some("2001:db8::7".parse::<IpAddr>().unwrap())
    );
}

#[test]
fn an_answer_without_an_address_gives_none() {
    for answer in [
        "",
        "<html><body>502 Bad Gateway</body></html>",
        "<Response><Ip></Ip></Response>",
        "<Response><Ip>not an address</Ip></Response>",
        "<Response><Ip><b>1.2.3.4</b></Ip></Response>",
        "<Response><Ip>1.2.3.4",
        "<Response><ip>1.2.3.4</ip></Response>",
        "<Response><Ip>1.2.3.4 <a href='https://example.org'>x</a></Ip></Response>",
    ] {
        assert_eq!(public_address(answer), None, "{answer}");
    }
}

#[test]
fn ping_and_traceroute_are_user_steps() {
    let host = HostName::new("garudalinux.org").unwrap();
    let ping = plan_ping(
        host.clone(),
        PingCount::new(4).unwrap(),
        PingTimeout::new(5).unwrap(),
    );

    let route = plan_traceroute(host, HopLimit::new(30).unwrap());

    assert_eq!(commands(&ping), ["ping -c 4 -W 5 -- garudalinux.org"]);
    assert_eq!(commands(&route), ["traceroute -m 30 -- garudalinux.org"]);
    assert!(
        ping.steps()
            .iter()
            .chain(route.steps())
            .all(|step| matches!(step.action, PlannedAction::User(_)))
    );
}

#[test]
fn a_missing_tool_is_queued_and_a_second_click_takes_it_back() {
    let snapshot = installed(&["networkmanager"]);
    let mut changes = ChangeSet::default();

    assert_eq!(
        package_state(&HOTSPOT_PACKAGE, &snapshot, &changes),
        PackageState::Missing
    );
    assert_eq!(
        toggle_install(&HOTSPOT_PACKAGE, &mut changes, &snapshot),
        PackageState::Queued
    );
    assert_eq!(changes.len(), 1);
    assert_eq!(
        toggle_install(&HOTSPOT_PACKAGE, &mut changes, &snapshot),
        PackageState::Missing
    );
    assert!(changes.is_empty());
}

#[test]
fn an_installed_tool_queues_nothing() {
    let snapshot = installed(&[HOTSPOT_PACKAGE.as_str()]);
    let mut changes = ChangeSet::default();

    assert_eq!(
        toggle_install(&HOTSPOT_PACKAGE, &mut changes, &snapshot),
        PackageState::Installed
    );
    assert_eq!(
        toggle_install(&HOTSPOT_PACKAGE, &mut changes, &Snapshot::default()),
        PackageState::Unknown
    );
    assert!(changes.is_empty());
}

#[test]
fn the_tool_packages_have_valid_names() {
    assert_eq!(HOTSPOT_PACKAGE.as_str(), "linux-wifi-hotspot");
    assert_eq!(super::TRACEROUTE_PACKAGE.as_str(), "traceroute");
}
