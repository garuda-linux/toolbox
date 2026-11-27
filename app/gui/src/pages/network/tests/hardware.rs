use std::collections::BTreeSet;
use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::network::Connectivity;
use toolbox_core::{Snapshot, Toggle};

use super::{NetworkPage, Shown, facts, network, page, sent, tab};
use crate::command::{Command, NetworkCommand};
use crate::network::{Address, Hardware, Network, Read};
use crate::page_service::Services;
use crate::testing::{PageFixture, rect_of};

fn without_tools() -> Shown {
    let status = network().status().cloned();
    let hardware = Hardware {
        pci: Read::Unavailable,
        usb: Read::Unavailable,
    };

    let mut snapshot = Snapshot::default();
    snapshot.insert(facts());
    snapshot.insert(InstalledPackages(BTreeSet::new()));
    let mut harness = PageFixture {
        snapshot: Arc::new(snapshot),
        services: Services {
            network: Network::showing(
                status,
                Some(Connectivity::Full),
                Some(hardware),
                Address::NotAsked,
            ),
            ..Services::default()
        },
        ..PageFixture::default()
    }
    .harness(NetworkPage::default());

    harness.run();
    tab(&mut harness, "Hardware");
    harness
}

#[test]
fn without_both_tools_one_empty_state_names_both_and_queues_pciutils() {
    let mut harness = without_tools();

    assert_eq!(
        harness
            .query_all_by_label_contains("lspci is not installed")
            .count(),
        1
    );
    assert_eq!(
        harness
            .query_all_by_label_contains("lsusb is not installed")
            .count(),
        1
    );

    harness
        .get_by_role_and_label(Role::Button, "Queue pciutils")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(NetworkCommand::TogglePciutils {
            wish: Toggle::On
        })]
    );
}

#[test]
fn copy_all_ends_at_the_right_edge_as_the_line_buttons_do() {
    let mut harness = page();
    tab(&mut harness, "Hardware");

    let all = rect_of(&harness, Role::Button, "Copy All");
    let line = harness
        .query_all_by_label("Copy This Line")
        .next()
        .expect("a device row")
        .rect();

    assert!(
        (all.right() - line.right()).abs() < 1.0,
        "Copy all at {all:?} does not end where Copy this line at {line:?} does"
    );
}

#[test]
fn the_intro_is_inset_from_the_card_edge_as_the_lines_are() {
    let mut harness = page();
    tab(&mut harness, "Hardware");

    let intro = harness
        .get_by_label("Network devices as lspci and lsusb list them.")
        .rect();

    let line = harness
        .query_all_by_label("Wireless")
        .next()
        .expect("a device row")
        .rect();

    assert!(
        (intro.left() - line.left()).abs() < 1.0,
        "the intro starts at {} and the line at {}",
        intro.left(),
        line.left()
    );
}
