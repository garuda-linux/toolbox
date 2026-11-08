use std::fs;
use std::net::IpAddr;
use std::path::Path;

use tempfile::TempDir;
use toolbox_state::network::{
    Connectivity, DefaultRoute, DeviceKind, Radio, RadioAccess, RadioKind, RadioTarget,
};
use zbus::OwnedVariant;

use super::network_manager::connectivity_from_value;
use super::{NetworkPrograms, NetworkSource as _, SystemNetwork, parse_default_route};
use crate::ProbeError;
use crate::testing::fake_program;

const ROUTES: &str = r#"[{"dst":"default","gateway":"192.168.178.1","dev":"wlan0","proto":"dhcp","prefsrc":"192.168.178.42","metric":600,"flags":[]},{"dst":"default","gateway":"10.0.0.1","dev":"enp5s0","metric":700,"flags":[]}]"#;

fn missing() -> NetworkPrograms {
    NetworkPrograms {
        ip: "/nonexistent/ip".into(),
        rfkill: "/nonexistent/rfkill".into(),
        lspci: "/nonexistent/lspci".into(),
        lsusb: "/nonexistent/lsusb".into(),
    }
}

fn switch(root: &Path, name: &str, kind: &str, soft: &str, hard: &str) {
    let folder = root.join("sys/class/rfkill").join(name);

    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("type"), format!("{kind}\n")).unwrap();
    fs::write(folder.join("soft"), format!("{soft}\n")).unwrap();
    fs::write(folder.join("hard"), format!("{hard}\n")).unwrap();
}

fn system() -> (TempDir, SystemNetwork) {
    let root = tempfile::tempdir().unwrap();
    let network = SystemNetwork::new(root.path(), missing());

    (root, network)
}

#[test]
fn the_first_default_route_names_router_and_interface() {
    assert_eq!(
        parse_default_route(ROUTES),
        Ok(DefaultRoute {
            gateway: Some("192.168.178.1".parse::<IpAddr>().unwrap()),
            interface: Some("wlan0".to_owned()),
        })
    );
}

#[test]
fn no_default_route_is_an_answer_not_an_error() {
    assert_eq!(parse_default_route("[]\n"), Ok(DefaultRoute::default()));
}

/// A point-to-point link (a VPN, a mobile modem) has no gateway.
#[test]
fn a_route_without_a_gateway_keeps_its_interface() {
    assert_eq!(
        parse_default_route(r#"[{"dst":"default","dev":"wg0","scope":"link","flags":[]}]"#),
        Ok(DefaultRoute {
            gateway: None,
            interface: Some("wg0".to_owned()),
        })
    );
}

#[test]
fn text_that_is_no_address_or_interface_name_is_left_out() {
    let odd = r#"[{"gateway":"<b>router</b>","dev":"wlan0 and more words here"}]"#;

    assert_eq!(parse_default_route(odd), Ok(DefaultRoute::default()));
    assert_eq!(
        parse_default_route(r#"[{"gateway":"fe80::1","dev":"../x"}]"#),
        Ok(DefaultRoute {
            gateway: Some("fe80::1".parse::<IpAddr>().unwrap()),
            interface: None,
        })
    );
}

#[test]
fn output_that_is_no_route_list_fails() {
    assert!(matches!(
        parse_default_route("default via 192.168.1.1 dev wlan0"),
        Err(ProbeError::Failed(_))
    ));
}

#[test]
fn the_route_comes_from_the_ip_program() {
    let (_folder, ip) = fake_program(ROUTES, "", 0);
    let network = SystemNetwork::new(
        "/",
        NetworkPrograms {
            ip: ip.into(),
            ..missing()
        },
    );

    assert_eq!(
        network.default_route().unwrap().interface.as_deref(),
        Some("wlan0")
    );
}

#[test]
fn without_the_ip_program_the_route_is_unavailable() {
    let (_root, network) = system();

    assert_eq!(network.default_route(), Err(ProbeError::Unavailable));
}

#[test]
fn radios_come_from_the_kernels_switches() {
    let (root, network) = system();

    switch(root.path(), "rfkill0", "bluetooth", "0", "0");
    switch(root.path(), "rfkill1", "wlan", "1", "0");
    switch(root.path(), "rfkill2", "nfc", "0", "1");
    fs::create_dir_all(root.path().join("sys/class/rfkill/rfkill3")).unwrap();

    assert_eq!(
        network.radios(),
        Ok(vec![
            Radio {
                kind: Some(RadioKind::Bluetooth),
                soft: false,
                hard: false,
            },
            Radio {
                kind: Some(RadioKind::Wifi),
                soft: true,
                hard: false,
            },
            Radio {
                kind: None,
                soft: false,
                hard: true,
            },
        ])
    );
}

#[test]
fn a_machine_without_switches_has_no_radios() {
    let (_root, network) = system();

    assert_eq!(network.radios(), Ok(Vec::new()));
}

#[test]
fn access_follows_the_rfkill_device() {
    let (root, network) = system();

    assert_eq!(network.radio_access(), RadioAccess::Missing);

    fs::create_dir_all(root.path().join("dev")).unwrap();
    fs::write(root.path().join("dev/rfkill"), "").unwrap();
    assert_eq!(network.radio_access(), RadioAccess::Allowed);
    assert_eq!(fs::read(root.path().join("dev/rfkill")).unwrap(), b"");
}

#[test]
fn a_refusing_rfkill_reports_its_complaint() {
    let (_folder, rfkill) =
        fake_program("", "rfkill: cannot open /dev/rfkill: Permission denied", 1);

    let network = SystemNetwork::new(
        "/",
        NetworkPrograms {
            rfkill: rfkill.into(),
            ..missing()
        },
    );

    let Err(ProbeError::Failed(reason)) = network.set_radio(RadioTarget::Wifi, true) else {
        panic!("a refused switch counts as done");
    };

    assert!(reason.contains("Permission denied"), "{reason}");
}

#[test]
fn a_willing_rfkill_switches_and_a_missing_one_is_unavailable() {
    let (_folder, rfkill) = fake_program("", "", 0);
    let willing = SystemNetwork::new(
        "/",
        NetworkPrograms {
            rfkill: rfkill.into(),
            ..missing()
        },
    );

    let (_root, without) = system();

    assert_eq!(willing.set_radio(RadioTarget::All, false), Ok(()));
    assert_eq!(
        without.set_radio(RadioTarget::All, false),
        Err(ProbeError::Unavailable)
    );
}

#[test]
fn devices_come_from_lspci_and_lsusb() {
    let (_pci_folder, lspci) = fake_program(
        "02:00.0 Ethernet controller [0200]: Realtek RTL8125 [10ec:8125] (rev 05)\n00:1f.3 Audio device [0403]: Intel Audio [8086:7a50]\n",
        "",
        0,
    );

    let (_usb_folder, lsusb) = fake_program(
        "Bus 001 Device 004: ID 8087:0033 Intel Corp. AX211 Bluetooth\n",
        "",
        0,
    );

    let network = SystemNetwork::new(
        "/",
        NetworkPrograms {
            lspci: lspci.into(),
            lsusb: lsusb.into(),
            ..missing()
        },
    );

    let pci = network.pci_devices().unwrap();
    let usb = network.usb_devices().unwrap();

    assert_eq!(pci.len(), 1);
    assert_eq!(pci[0].kind, DeviceKind::Ethernet);
    assert_eq!(usb.len(), 1);
    assert_eq!(usb[0].kind, DeviceKind::Bluetooth);
}

#[test]
fn without_the_list_programs_the_devices_are_unavailable() {
    let (_root, network) = system();

    assert_eq!(network.pci_devices(), Err(ProbeError::Unavailable));
    assert_eq!(network.usb_devices(), Err(ProbeError::Unavailable));
}

#[test]
fn the_connectivity_property_is_read_as_a_number_only() {
    assert_eq!(
        connectivity_from_value(&OwnedVariant::from(4_u32)),
        Connectivity::Full
    );
    assert_eq!(
        connectivity_from_value(&OwnedVariant::from(2_u32)),
        Connectivity::Portal
    );
    assert_eq!(
        connectivity_from_value(&OwnedVariant::from(true)),
        Connectivity::Unknown
    );
}

#[test]
fn an_unreadable_device_denies_radio_access() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("dev"), "").unwrap();
    let network = SystemNetwork::new(dir.path(), missing());

    assert_eq!(network.radio_access(), RadioAccess::Denied);
}
