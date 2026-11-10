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
use crate::test_support::fake_program;

/// The Lima VM with a `wg0` link added after its DHCP route.
const ROUTES: &str = include_str!("fixtures/ip-route-default-two.json");
const LAPTOP_ROUTE: &str = include_str!("fixtures/bare-metal/ip-route-default.json");
const LAPTOP_RFKILL: &str = include_str!("fixtures/bare-metal/rfkill");
const LAPTOP_LSPCI: &str = crate::testing::network::BARE_METAL_LSPCI_NN;
const LAPTOP_LSUSB: &str = crate::testing::network::BARE_METAL_LSUSB;
const VM_LSPCI: &str = crate::testing::network::LSPCI_NN;
const VM_LSUSB: &str = crate::testing::network::LSUSB_STDOUT;

/// What `ip -json route show default dev lo` printed on the laptop: the
/// answer of a machine without a default route.
const NO_ROUTE: &str = "[]\n";

/// What `rfkill list` printed in a sandbox without `/dev/rfkill`.
const RFKILL_REFUSAL: &str = "rfkill: cannot open /dev/rfkill: No such file or directory\n";

fn missing() -> NetworkPrograms {
    NetworkPrograms {
        ip: "/nonexistent/ip".into(),
        rfkill: "/nonexistent/rfkill".into(),
        lspci: "/nonexistent/lspci".into(),
        lsusb: "/nonexistent/lsusb".into(),
    }
}

/// Writes the switches of a capture with lines like
/// `rfkill0 type=bluetooth soft=0 hard=0 name=hci0` below `root`, the way
/// sysfs shows them: one file per value.
fn switches(root: &Path, capture: &str) {
    for line in capture.lines() {
        let mut fields = line.split_whitespace();
        let folder = root.join("sys/class/rfkill").join(fields.next().unwrap());

        fs::create_dir_all(&folder).unwrap();

        for field in fields {
            let (file, value) = field.split_once('=').unwrap();
            fs::write(folder.join(file), format!("{value}\n")).unwrap();
        }
    }
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
            gateway: Some("192.168.5.2".parse::<IpAddr>().unwrap()),
            interface: Some("eth0".to_owned()),
        })
    );
}

#[test]
fn no_default_route_is_an_answer_not_an_error() {
    assert_eq!(parse_default_route(NO_ROUTE), Ok(DefaultRoute::default()));
}

/// A point-to-point link (a VPN, a mobile modem) has no gateway. The test
/// drops the DHCP route of the capture, so its `wg0` route comes first.
#[test]
fn a_route_without_a_gateway_keeps_its_interface() {
    let (_dhcp, wireguard) = ROUTES.split_once("},").unwrap();

    assert_eq!(
        parse_default_route(&format!("[{wireguard}")),
        Ok(DefaultRoute {
            gateway: None,
            interface: Some("wg0".to_owned()),
        })
    );
}

/// The laptop's route with its gateway and interface replaced by text no
/// router or interface would have.
#[test]
fn text_that_is_no_address_or_interface_name_is_left_out() {
    let odd = LAPTOP_ROUTE
        .replacen("192.168.178.1", "<b>router</b>", 1)
        .replacen("wlan0", "wlan0 and more words here", 1);

    let link_local = LAPTOP_ROUTE
        .replacen("192.168.178.1", "fe80::1", 1)
        .replacen("wlan0", "../x", 1);

    assert_eq!(parse_default_route(&odd), Ok(DefaultRoute::default()));
    assert_eq!(
        parse_default_route(&link_local),
        Ok(DefaultRoute {
            gateway: Some("fe80::1".parse::<IpAddr>().unwrap()),
            interface: None,
        })
    );
}

/// The capture cut in half, as a killed `ip` would leave it.
#[test]
fn output_that_is_no_route_list_fails() {
    assert!(matches!(
        parse_default_route(LAPTOP_ROUTE.get(..LAPTOP_ROUTE.len() / 2).unwrap()),
        Err(ProbeError::Failed(_))
    ));
}

#[test]
fn the_route_comes_from_the_ip_program() {
    let (_folder, ip) = fake_program(LAPTOP_ROUTE, "", 0);
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

    switches(root.path(), LAPTOP_RFKILL);

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
                soft: false,
                hard: false,
            },
        ])
    );
}

/// The laptop's switches with Wi-Fi blocked by software, plus an NFC
/// switch blocked by hardware (the laptop has none) and a switch folder
/// without files.
#[test]
fn blocked_unknown_and_empty_switches_are_read_as_such() {
    let (root, network) = system();
    let edited = LAPTOP_RFKILL.replacen("type=wlan soft=0", "type=wlan soft=1", 1)
        + "rfkill2 type=nfc soft=0 hard=1 name=nfc0\n";

    switches(root.path(), &edited);
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

/// The Lima VM's `/sys/class/rfkill` is an empty folder.
#[test]
fn a_machine_without_switches_has_no_radios() {
    let (root, network) = system();

    assert_eq!(network.radios(), Ok(Vec::new()));

    fs::create_dir_all(root.path().join("sys/class/rfkill")).unwrap();
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
    let (_folder, rfkill) = fake_program("", RFKILL_REFUSAL, 1);

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

    assert!(reason.contains("cannot open /dev/rfkill"), "{reason}");
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

/// A root whose `/sys/bus/usb/devices` is the laptop's capture, linked
/// like the entries of the real folder are.
fn laptop_sysfs() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    let bus = root.path().join("sys/bus/usb");

    fs::create_dir_all(&bus).unwrap();
    std::os::unix::fs::symlink(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/network/fixtures/bare-metal/sys-bus-usb-devices"),
        bus.join("devices"),
    )
    .unwrap();
    root
}

fn devices(root: &Path, lspci: &str, lsusb: &str) -> Vec<(DeviceKind, String)> {
    let (_pci_folder, lspci) = fake_program(lspci, "", 0);
    let (_usb_folder, lsusb) = fake_program(lsusb, "", 0);
    let network = SystemNetwork::new(
        root,
        NetworkPrograms {
            lspci: lspci.into(),
            lsusb: lsusb.into(),
            ..missing()
        },
    );

    network
        .pci_devices()
        .unwrap()
        .into_iter()
        .chain(network.usb_devices().unwrap())
        .map(|device| (device.kind, device.description))
        .collect()
}

/// Plain `lsusb` prints no class. The interface class in sysfs shows the
/// Bluetooth half of the MT7922 (`0489:e0f6`, named only
/// `Wireless_Device`).
#[test]
fn devices_come_from_lspci_lsusb_and_the_usb_interfaces() {
    let root = laptop_sysfs();

    assert_eq!(
        devices(root.path(), LAPTOP_LSPCI, LAPTOP_LSUSB),
        [
            (
                DeviceKind::Ethernet,
                "Realtek Semiconductor Co., Ltd. RTL8111/8168/8211/8411 PCI Express Gigabit Ethernet Controller [10ec:8168] (rev 15)".to_owned()
            ),
            (
                DeviceKind::Wireless,
                "MEDIATEK Corp. MT7922 802.11ax PCI Express Wireless Network Adapter [14c3:0616]".to_owned()
            ),
            (
                DeviceKind::Ethernet,
                "Realtek Semiconductor Corp. RTL8153 Gigabit Ethernet Adapter".to_owned()
            ),
            (
                DeviceKind::Bluetooth,
                "Foxconn / Hon Hai Wireless_Device".to_owned()
            ),
        ]
    );
}

/// The VM has no capture of its USB folder, so its root has none.
#[test]
fn a_virtual_machine_lists_its_virtio_card_only() {
    let root = tempfile::tempdir().unwrap();

    assert_eq!(
        devices(root.path(), VM_LSPCI, VM_LSUSB),
        [(
            DeviceKind::Ethernet,
            "Red Hat, Inc. Virtio network device [1af4:1000]".to_owned()
        )]
    );
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
