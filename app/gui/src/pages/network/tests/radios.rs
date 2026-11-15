use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::network::Connectivity;

use super::{network, page_showing};
use crate::network::{Address, Network, Read};

fn without_radios() -> Network {
    let mut status = network().status().cloned().unwrap();
    status.radios = Read::Known(Vec::new());
    Network::showing(
        Some(status),
        Some(Connectivity::Full),
        None,
        Address::NotAsked,
    )
}

#[test]
fn an_absent_radio_says_so_without_a_dead_switch() {
    let harness = page_showing(network());

    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "Bluetooth")
            .is_none()
    );
    assert!(
        harness
            .query_by_label("No Bluetooth adapter found")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "Wi-Fi")
            .is_some()
    );
}

#[test]
fn without_any_radio_flight_mode_has_no_switch() {
    let harness = page_showing(without_radios());

    assert!(
        harness
            .query_by_role_and_label(Role::CheckBox, "Flight mode")
            .is_none()
    );
    assert!(harness.query_by_label("No radio to switch off").is_some());
}
