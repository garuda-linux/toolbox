use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::runtime::Runtime;
use toolbox_core::features::network::{
    Connectivity, DefaultRoute, Device, Radio, RadioAccess, RadioKind, RadioTarget,
};
use toolbox_https::RequestError;
use toolbox_system::{AddressSource, Checked, NetworkActions, NetworkSource, ProbeError};

use super::{Address, Network, NetworkSetup, Read};
use crate::testing::until;

#[derive(Default)]
struct Machine {
    switches: AtomicUsize,
    routes: AtomicUsize,
}

struct Service(bool);

struct Actions(Arc<Machine>);

impl NetworkSource for Machine {
    fn default_route(&self) -> Result<DefaultRoute, ProbeError> {
        self.routes.fetch_add(1, Ordering::SeqCst);
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
        self.switches.fetch_add(1, Ordering::SeqCst);
        Err(ProbeError::Failed("denied".to_owned()))
    }

    fn pci_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Err(ProbeError::Unavailable)
    }

    fn usb_devices(&self) -> Result<Vec<Device>, ProbeError> {
        Ok(Vec::new())
    }
}

impl AddressSource for Service {
    fn answer(&self) -> Result<String, RequestError> {
        if self.0 {
            Ok("<Response><Ip>203.0.113.7</Ip></Response>".to_owned())
        } else {
            Err(RequestError::NoAnswer("offline".to_owned()))
        }
    }
}

impl NetworkActions for Actions {
    fn connectivity(&self, _recheck: bool) -> Checked {
        Box::pin(async { Connectivity::Full })
    }

    fn set_radio(&self, target: RadioTarget, blocked: bool) -> Result<(), ProbeError> {
        self.0.set_radio(target, blocked)
    }
}

fn network(runtime: &Runtime, machine: &Arc<Machine>, online: bool) -> Network {
    Network::new(
        Some(NetworkSetup {
            runtime: runtime.handle().clone(),
            source: Arc::<Machine>::clone(machine),
            actions: Arc::new(Actions(Arc::clone(machine))),
            address: Arc::new(Service(online)),
        }),
        egui::Context::default(),
    )
}

fn settle(network: &mut Network, done: impl Fn(&Network) -> bool) {
    until("the read never ended", || {
        network.take();
        done(network)
    });
}

#[test]
fn nothing_is_read_before_the_page_opens() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut network = network(&runtime, &machine, true);

    network.plan_finished();

    // A wrong read would run on a runtime thread, so it gets the time to
    // land before the count.
    std::thread::sleep(Duration::from_millis(50));
    network.take();

    assert_eq!(machine.routes.load(Ordering::SeqCst), 0);
    assert_eq!(network.status(), None);
    assert_eq!(network.connectivity(), None);
}

#[test]
fn opening_reads_the_status_the_internet_and_the_hardware() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut network = network(&runtime, &machine, true);

    network.open();
    settle(&mut network, |shown| {
        shown.status().is_some() && shown.connectivity().is_some() && shown.hardware().is_some()
    });

    let status = network.status().unwrap();
    assert_eq!(
        status
            .route
            .known()
            .and_then(|route| route.interface.as_deref()),
        Some("wlan0")
    );
    assert_eq!(network.connectivity(), Some(Connectivity::Full));
    let hardware = network.hardware().unwrap();
    assert_eq!(
        (&hardware.pci, &hardware.usb),
        (&Read::Unavailable, &Read::Known(Vec::new()))
    );
}

#[test]
fn the_address_is_asked_only_on_request_and_can_be_forgotten() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut network = network(&runtime, &machine, true);

    assert_eq!(*network.address(), Address::NotAsked);

    network.lookup_address();
    assert_eq!(*network.address(), Address::Loading);
    settle(&mut network, |shown| *shown.address() != Address::Loading);

    let known: IpAddr = "203.0.113.7".parse().unwrap();
    assert_eq!(*network.address(), Address::Known(known));

    network.forget_address();
    assert_eq!(*network.address(), Address::NotAsked);
}

#[test]
fn an_unreachable_service_leaves_the_address_unknown() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut network = network(&runtime, &machine, false);

    network.lookup_address();
    settle(&mut network, |shown| *shown.address() != Address::Loading);

    assert_eq!(*network.address(), Address::Failed);
}

#[test]
fn a_refused_radio_switch_is_remembered() {
    let runtime = Runtime::new().unwrap();
    let machine = Arc::new(Machine::default());
    let mut network = network(&runtime, &machine, true);

    network.set_radio(RadioTarget::Wifi, true);
    settle(&mut network, Network::radio_failed);

    assert_eq!(machine.switches.load(Ordering::SeqCst), 1);
}
