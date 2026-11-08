mod internet;
mod services;
mod status_read;
mod tools;
mod unload;

use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::network::{
    BlacklistEntry, Category, Connectivity, DefaultRoute, Device, DeviceKind, ModuleFacts,
    NetworkModule, Radio, RadioAccess, RadioKind, RadioTarget,
};
use toolbox_gui_kit::widgets::ButtonKind;
use toolbox_plan::{HostName, KernelModule, PingCount, PingTimeout};

use super::NetworkPage;
use crate::command::{Command, NetworkCommand};
use crate::network::{Address, Hardware, Network, Read, StatusRead};
use crate::pages::{NetworkRefusal, Refusals};
use crate::testing::{PageFixture, PageState, assert_header, button_kind, rect_of};

type Shown = Harness<'static, PageState<NetworkPage>>;

fn module(name: &str) -> KernelModule {
    KernelModule::new(name).unwrap()
}

/// `e1000e` loaded, `iwlwifi` blacklisted by the app, `r8169` by a
/// package's file.
fn facts() -> ModuleFacts {
    ModuleFacts {
        loaded: BTreeSet::from([module("e1000e")]),
        modules: vec![
            NetworkModule {
                name: module("e1000e"),
                category: Category::Ethernet,
            },
            NetworkModule {
                name: module("iwlwifi"),
                category: Category::Wireless,
            },
            NetworkModule {
                name: module("r8169"),
                category: Category::Ethernet,
            },
        ],
        blacklists: vec![
            BlacklistEntry {
                module: module("iwlwifi"),
                file: "/etc/modprobe.d/garuda-toolbox-blacklist.conf".to_owned(),
                own: true,
            },
            BlacklistEntry {
                module: module("r8169"),
                file: "/usr/lib/modprobe.d/vendor.conf".to_owned(),
                own: false,
            },
        ],
    }
}

fn network() -> Network {
    network_with(Address::NotAsked)
}

fn network_with(address: Address) -> Network {
    let status = StatusRead {
        route: Read::Known(DefaultRoute {
            gateway: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            interface: Some("enp3s0".to_owned()),
        }),
        radios: Read::Known(vec![Radio {
            kind: Some(RadioKind::Wifi),
            soft: false,
            hard: false,
        }]),
        access: RadioAccess::Allowed,
    };

    let hardware = Hardware {
        pci: Read::Known(vec![Device {
            kind: DeviceKind::Ethernet,
            description: "Intel Ethernet Connection I219-V".to_owned(),
            raw: "00:1f.6 Ethernet controller [0200]: Intel Ethernet Connection I219-V".to_owned(),
        }]),
        usb: Read::Unavailable,
    };

    Network::showing(
        Some(status),
        Some(Connectivity::Full),
        Some(hardware),
        address,
    )
}

fn page(refusals: Refusals) -> Shown {
    page_showing(network(), refusals)
}

fn page_showing(network: Network, refusals: Refusals) -> Shown {
    let mut snapshot = Snapshot::default();
    snapshot.insert(facts());
    snapshot.insert(InstalledPackages(BTreeSet::new()));
    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        network,
        refusals,
        ..PageFixture::default()
    }
    .harness(NetworkPage::default());

    harness.run();
    harness
}

fn tab(harness: &mut Shown, name: &str) {
    harness
        .get_by_role_and_label(Role::RadioButton, name)
        .click();
    harness.run();
}

fn sent(harness: &Shown) -> Vec<Command> {
    harness.state().fixture.sent()
}

#[test]
fn the_status_tab_shows_the_route_and_asks_for_the_address_on_a_click() {
    let mut harness = page(Refusals::default());

    assert!(harness.query_by_label("192.168.1.1").is_some());
    assert!(harness.query_by_label("enp3s0").is_some());

    harness.get_by_label("Show public address").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(NetworkCommand::LookupAddress)]
    );
}

#[test]
fn a_known_address_shows_copy_and_keeps_hide_in_the_menu() {
    let known = Address::Known(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)));
    let mut harness = page_showing(network_with(known), Refusals::default());

    assert!(harness.query_by_label("Copy").is_some());
    assert!(harness.query_by_label("Hide").is_none());
    harness
        .get_by_label("More actions for Public address")
        .click();
    harness.run();
    harness.get_by_label("Hide").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(NetworkCommand::ForgetAddress)]
    );
}

#[test]
fn the_wifi_switch_blocks_the_radio() {
    let mut harness = page(Refusals::default());

    harness
        .get_by_role_and_label(Role::CheckBox, "Wi-Fi")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(NetworkCommand::SetRadio {
            target: RadioTarget::Wifi,
            blocked: true,
        })]
    );
}

#[test]
fn a_driver_the_app_blacklists_can_be_taken_off() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Drivers");

    assert!(harness.query_by_label("Showing 3 of 3 drivers").is_some());

    assert!(
        harness
            .query_by_label("Queue removal from blacklist")
            .is_none()
    );
    harness.get_by_label("More actions for iwlwifi").click();
    harness.run();
    harness.get_by_label("Queue removal from blacklist").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(NetworkCommand::Blacklist {
            module: module("iwlwifi"),
            on: false,
        })]
    );
}

#[test]
fn a_driver_another_file_blacklists_has_no_blacklist_button() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Drivers");

    assert!(
        harness
            .query_by_label_contains("Blacklisted by /usr/lib/modprobe.d/vendor.conf")
            .is_some()
    );
    assert!(harness.query_by_label("More actions for r8169").is_none());
    assert!(harness.query_by_label("More actions for e1000e").is_some());
}

#[test]
fn a_search_without_a_match_says_so() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Drivers");

    harness.get_by_label("Search drivers").click();
    harness.run();
    harness.get_by_label("Search drivers").type_text("klingon");
    harness.run();

    assert!(harness.query_by_label("No driver matches").is_some());
}

#[test]
fn the_hardware_tab_lists_the_devices_and_says_what_is_missing() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Hardware");

    assert!(
        harness
            .query_by_label("Intel Ethernet Connection I219-V")
            .is_some()
    );
    assert!(
        harness
            .query_by_label_contains("lsusb is not installed")
            .is_some()
    );
}

#[test]
fn ping_runs_with_the_default_host() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Tools");

    harness.get_by_label("Run ping").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(NetworkCommand::Ping {
            host: HostName::new("garudalinux.org").unwrap(),
            count: PingCount::new(4).unwrap(),
            timeout: PingTimeout::new(5).unwrap(),
        })]
    );
}

#[test]
fn traceroute_waits_for_its_package() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Tools");

    assert!(
        harness
            .query_by_label_contains("needs the package traceroute")
            .is_some()
    );

    harness.get_by_label("Run traceroute").click();
    harness.run();

    assert_eq!(sent(&harness), []);
}

#[test]
fn a_refused_blacklist_names_the_driver_until_dismissed() {
    let refusals = Refusals {
        network: Some(NetworkRefusal {
            code: "other-file",
            driver: "r8169".to_owned(),
        }),
        ..Refusals::default()
    };

    let mut harness = page(refusals);

    assert!(harness.query_by_label_contains("r8169: another").is_some());

    harness.get_by_label("Dismiss").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(NetworkCommand::DismissRefusal)]
    );
}

#[test]
fn the_header_stays_beside_the_title_and_drivers_get_a_toolbar() {
    let mut harness = page(Refusals::default());
    let texts = (
        "Network",
        "The connection, the radios, the network drivers and tests.",
    );

    let status = rect_of(&harness, Role::RadioButton, "Status");
    assert_header(&harness, texts, Some(status), None);

    tab(&mut harness, "Drivers");

    assert_eq!(rect_of(&harness, Role::RadioButton, "Status"), status);
    let search = rect_of(&harness, Role::TextInput, "Search drivers");
    assert_header(&harness, texts, Some(status), Some(search));
}

#[test]
fn chip_rows_filter_the_drivers_by_state_and_kind() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Drivers");
    assert!(harness.query_by_role(Role::ComboBox).is_none());

    harness
        .get_by_role_and_label(Role::Group, "State")
        .get_by_label("Loaded")
        .click();
    harness.run();
    assert!(harness.query_by_label("Showing 1 of 3 drivers").is_some());

    harness
        .get_by_role_and_label(Role::Group, "State")
        .get_by_label("All")
        .click();
    harness
        .get_by_role_and_label(Role::Group, "Kind")
        .get_by_label("Wireless")
        .click();
    harness.run();
    assert!(harness.query_by_label("Showing 1 of 3 drivers").is_some());
}

#[test]
fn ping_is_no_stronger_than_the_other_tools() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Tools");

    assert_eq!(button_kind(&harness, "Run ping"), ButtonKind::Soft);
}

#[test]
fn a_device_copy_button_looks_like_a_button() {
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Hardware");

    let copy = harness
        .query_all_by_label("Copy this line")
        .next()
        .expect("a device row")
        .rect()
        .center();

    let fill = toolbox_gui_kit::testing::fill_under(&harness, copy);
    assert_eq!(
        fill,
        Some(toolbox_gui_kit::tokens::current(&harness.ctx).field)
    );
}
