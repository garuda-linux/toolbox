use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use futures_lite::future::Boxed;
use tokio::runtime::Runtime;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::network::{
    BlacklistEntry, Category, Connectivity, DefaultRoute, Device, ModuleFacts, NetworkModule,
    Radio, RadioAccess, RadioKind, RadioTarget, pci_devices,
};
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::network::{DryRunAddress, NetworkSetup};
use toolbox_gui::queue::Queue;
use toolbox_gui::units::{self, UnitList, UnitLister};
use toolbox_gui::{App, Setup, snapshot};
use toolbox_plan::{KernelModule, UnitName, UnitScope};
use toolbox_system::{Checked, NetworkActions, NetworkSource, ProbeError};

use crate::common::{Fixture, SMALLEST, TALL, WIDTH, open_tab, packages, window};
use crate::system::{first_read, settle, step_until};

const LAPTOP_LSPCI: &str =
    include_str!("../../../../crates/system/src/network/fixtures/bare-metal/lspci-nn");

/// One Wi-Fi radio, a router, the PCI cards of a bare-metal laptop and no
/// lsusb.
struct Machine;

/// The internet is there, and every switch works.
struct Actions;

/// `NetworkManager` running, Bluetooth stopped, `GeoClue` on demand.
struct Units;

impl NetworkSource for Machine {
    fn default_route(&self) -> Result<DefaultRoute, ProbeError> {
        Ok(DefaultRoute {
            gateway: Some("192.168.1.1".parse().unwrap()),
            interface: Some("wlan0".to_owned()),
        })
    }

    fn radios(&self) -> Result<Vec<Radio>, ProbeError> {
        Ok(vec![Radio {
            kind: Some(RadioKind::Wifi),
            soft: false,
            hard: false,
        }])
    }

    fn radio_access(&self) -> RadioAccess {
        RadioAccess::Allowed
    }

    fn set_radio(&self, _target: RadioTarget, _blocked: bool) -> Result<(), ProbeError> {
        Ok(())
    }

    fn pci_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Ok(pci_devices(LAPTOP_LSPCI))
    }

    fn usb_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Err(ProbeError::Unavailable)
    }
}

impl UnitLister for Units {
    fn list(&self, scope: UnitScope) -> Boxed<UnitList> {
        Box::pin(async move {
            let units = match scope {
                UnitScope::System => vec![
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
                    unit("geoclue.service", ActiveState::Inactive, FileState::Static),
                ],
                UnitScope::User => Vec::new(),
            };

            Ok(Arc::new(units))
        })
    }
}

impl NetworkActions for Actions {
    fn connectivity(&self, _recheck: bool) -> Checked {
        Box::pin(async { Connectivity::Full })
    }

    fn set_radio(&self, _target: RadioTarget, _blocked: bool) -> Result<(), ProbeError> {
        Ok(())
    }
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

fn module(name: &str) -> KernelModule {
    KernelModule::new(name).unwrap()
}

/// `iwlwifi` loaded, `r8169` offered, traceroute installed.
fn system() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(ModuleFacts {
        loaded: BTreeSet::from([module("iwlwifi")]),
        modules: vec![
            NetworkModule {
                name: module("iwlwifi"),
                category: Category::Wireless,
            },
            NetworkModule {
                name: module("r8169"),
                category: Category::Ethernet,
            },
        ],
        blacklists: vec![BlacklistEntry {
            module: module("pcspkr"),
            file: "/usr/lib/modprobe.d/quiet.conf".to_owned(),
            own: false,
        }],
    });
    snapshot.insert(InstalledPackages(packages(&["traceroute"])));
    snapshot
}

pub fn network() -> Fixture {
    let runtime = Runtime::new().unwrap();
    let handle = runtime.handle().clone();
    let probed = system();
    let mut harness = window(TALL).build_eframe(move |cc| {
        let probe = move || (probed.clone(), Vec::new());
        let units = units::spawn(
            &handle,
            toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
            Arc::new(Units),
            Duration::ZERO,
        );

        App::new(
            cc,
            Setup {
                queue: Queue::in_memory(ChangeSet::default()),
                snapshot: Some(snapshot::spawn(
                    &handle,
                    toolbox_gui::waking::repaint(cc.egui_ctx.clone()),
                    probe,
                )),
                units: Some(units),
                network: Some(NetworkSetup {
                    runtime: handle.clone(),
                    source: Arc::new(Machine),
                    actions: Arc::new(Actions),
                    address: Arc::new(DryRunAddress),
                }),
                ..Setup::default()
            },
        )
    });

    first_read(&mut harness);
    harness.get_by_role_and_label(Role::Tab, "Network").click();
    step_until(&mut harness, "the page's reads", |harness| {
        harness.query_by_label("wlan0").is_some() && harness.query_by_label("Connected").is_some()
    });
    settle(&mut harness);

    Fixture::new(harness, runtime)
}

#[test]
fn the_public_address_shows_after_a_click() {
    let mut fixture = network();
    let harness = fixture.harness();

    assert!(harness.query_by_label("192.168.1.1").is_some());

    harness.get_by_label("Show public address").click();
    step_until(harness, "the address", |harness| {
        harness.query_by_label_contains("203.0.113.7").is_some()
    });
}

#[test]
fn a_blacklisted_driver_waits_in_the_queue() {
    let mut fixture = network();
    let harness = fixture.harness();
    open_tab(harness, "Drivers");

    harness.get_by_label("More actions for r8169").click();
    harness.run();
    harness.get_by_label("Queue blacklist").click();
    harness.run();

    assert!(
        harness
            .query_by_label("Queued: add to the blacklist")
            .is_some()
    );
    assert!(harness.query_by_label("change queued").is_some());
}

#[test]
fn a_driver_without_a_note_sits_level_with_its_button() {
    let mut fixture = network();
    let harness = fixture.harness();
    open_tab(harness, "Drivers");

    // The row's name, not the row itself, which carries the same label.
    let name = harness
        .query_all_by_label("r8169")
        .map(|node| node.rect())
        .min_by(|left, right| left.width().total_cmp(&right.width()))
        .unwrap();

    let button = harness.get_by_label("Load").rect();

    assert!(
        (name.center().y - button.center().y).abs() < 1.5,
        "name at {name:?}, button at {button:?}"
    );
}

#[test]
fn a_long_device_name_wraps_above_its_kind_at_every_width() {
    let mut fixture = network();
    let harness = fixture.harness();
    open_tab(harness, "Hardware");

    for width in [SMALLEST.0, 800.0, WIDTH] {
        harness.set_size(egui::vec2(width, TALL));
        settle(harness);

        for (name, kind) in [("RTL8111/8168", "Ethernet"), ("MT7922", "Wireless")] {
            // The row's name, not the row itself, which carries the same label.
            let name = harness
                .query_all_by_label_contains(name)
                .map(|node| node.rect())
                .min_by(|left, right| left.width().total_cmp(&right.width()))
                .unwrap();

            let kind = harness.get_by_label(kind).rect();
            assert!(
                kind.top() >= name.bottom(),
                "width {width}: name at {name:?}, kind at {kind:?}"
            );
        }
    }
}
