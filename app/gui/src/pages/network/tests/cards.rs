use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::network::InterfaceCard;

use super::{Address, Shown, network_with, page_showing};
use crate::network::samples;

fn with_cards(cards: Vec<InterfaceCard>) -> Shown {
    page_showing(network_with(Address::NotAsked).with_interfaces(cards))
}

#[test]
fn each_activated_device_has_a_card_titled_by_its_connection() {
    let harness = with_cards(samples::cards());

    // The title and the network name of the Wi-Fi row.
    assert_eq!(harness.query_all_by_label_contains("network-3").count(), 2);
    assert!(
        harness
            .query_by_label_contains("Wired connection 1")
            .is_some()
    );
}

#[test]
fn the_wifi_card_shows_its_addresses_servers_mac_and_signal() {
    let harness = with_cards(vec![samples::wifi()]);

    for value in [
        "192.168.178.30/24",
        "fe80::d/64",
        "192.168.178.1",
        "52:54:00:00:00:07",
        "Good (57 %)",
    ] {
        assert!(harness.query_by_label(value).is_some(), "no {value} row");
    }

    // The Connection card's interface row and the card's own row.
    assert_eq!(harness.query_all_by_label_contains("wlan0").count(), 2);
    assert!(harness.query_by_label("1000 Mb/s").is_none());
}

#[test]
fn the_ethernet_card_shows_its_speed_and_no_network_name() {
    let harness = with_cards(vec![samples::ethernet()]);

    assert!(harness.query_by_label("1000 Mb/s").is_some());
    assert!(harness.query_by_label("SSID").is_none());
    assert!(harness.query_by_label("Good (57 %)").is_none());
}

#[test]
fn the_wifi_values_have_copy_buttons_but_the_signal_does_not() {
    let harness = with_cards(vec![samples::wifi()]);

    for label in [
        "Copy Interface",
        "Copy IPv4",
        "Copy MAC",
        "Copy DNS",
        "Copy SSID",
    ] {
        assert!(
            harness
                .query_all_by_role_and_label(Role::Button, label)
                .next()
                .is_some(),
            "no {label} button"
        );
    }

    assert!(
        harness
            .query_all_by_role_and_label(Role::Button, "Copy Signal")
            .next()
            .is_none()
    );
}

#[test]
fn without_networkmanager_no_card_shows() {
    let harness = page_showing(network_with(Address::NotAsked));

    // Only the Connection card's interface row names wlan0.
    assert_eq!(harness.query_all_by_label_contains("wlan0").count(), 1);
    assert!(harness.query_by_label("MAC").is_none());
}

#[test]
fn the_cards_look_as_designed() {
    let mut harness = with_cards(samples::cards());

    harness.snapshot("network/cards_dark");

    harness.get_by_label("1000 Mb/s").scroll_to_me();
    harness.run();
    harness.snapshot("network/cards_wired_dark");
}
