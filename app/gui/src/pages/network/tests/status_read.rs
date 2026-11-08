use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::network::{Connectivity, RadioAccess};

use super::{Refusals, network, page_showing};
use crate::network::{Address, Network, Read, StatusRead};

fn showing(change: impl FnOnce(&mut StatusRead)) -> Network {
    let mut status = network().status().cloned().unwrap();
    change(&mut status);
    Network::showing(
        Some(status),
        Some(Connectivity::Full),
        None,
        Address::NotAsked,
    )
}

#[test]
fn a_route_that_could_not_be_read_is_not_known_not_absent() {
    let harness = page_showing(
        showing(|status| status.route = Read::Failed),
        Refusals::default(),
    );

    assert!(harness.query_all_by_label("Not known").next().is_some());
    assert!(harness.query_by_label("Not available").is_none());
}

#[test]
fn without_the_radio_device_a_switch_does_not_claim_on() {
    let harness = page_showing(
        showing(|status| status.access = RadioAccess::Missing),
        Refusals::default(),
    );

    assert!(
        harness
            .query_all_by_label("Wi-Fi")
            .all(|node| node.accesskit_node().toggled() != Some(egui::accesskit::Toggled::True)),
        "Wi-Fi claims to be on"
    );
    assert!(
        harness
            .query_all_by_label("State not known")
            .next()
            .is_some()
    );
}
