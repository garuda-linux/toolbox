//! The Network page's reads and acts: the route and the radios, the
//! internet check, the device lists, the public address and the radio
//! switches. Each runs on the runtime and wakes the frame with its result.

use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;

use crossbeam_channel::Receiver;
use tokio::runtime::Handle;
use toolbox_core::features::network::{Connectivity, InterfaceCard, RadioTarget};
use toolbox_https::RequestError;
use toolbox_services::address::{AddressProblem, AddressSource, lookup_address};
use toolbox_system::{
    CallError, Cards, Checked, NetworkActions, NetworkSource, ProbeError, connectivity,
};

use super::{Hardware, Read, StatusRead, live};
use crate::lane::Lane;
use crate::waking::{self, Waking};

/// An address no machine has (TEST-NET-3 of RFC 5737), so a self-test
/// screenshot never shows a real one.
const STAND_IN_ANSWER: &str = "<Response><Ip>203.0.113.7</Ip><Status>OK</Status></Response>";

pub struct NetworkSetup {
    pub runtime: Handle,
    pub source: Arc<dyn NetworkSource>,
    pub actions: Arc<dyn NetworkActions>,
    pub address: Arc<dyn AddressSource>,
}

/// Stands in for the address service in a dry run: no request leaves
/// the machine.
#[derive(Clone, Copy, Debug)]
pub struct DryRunAddress;

/// Reads the last internet check and switches no radio, for the dry run
/// and its self-test.
#[derive(Clone, Copy, Debug)]
pub struct DryRunActions;

pub enum Landed {
    Status(StatusRead),
    Connectivity(Connectivity),
    Hardware(Hardware),
    /// `None` hides the cards, as NetworkManager does not run or failed.
    Interfaces(Option<Vec<InterfaceCard>>),
    Address(Result<IpAddr, AddressProblem>),
    Radio(Result<(), ProbeError>),
}

#[derive(Debug)]
pub struct Jobs {
    setup: NetworkSetup,
    lane: Lane<Landed>,
    /// The live watcher's answers, which wake the frame as the lane's do.
    answers: Waking<Landed>,
    live: Receiver<Landed>,
}

impl AddressSource for DryRunAddress {
    fn answer(&self) -> Result<String, RequestError> {
        Ok(STAND_IN_ANSWER.to_owned())
    }
}

impl NetworkActions for DryRunActions {
    fn connectivity(&self, _recheck: bool) -> Checked {
        Box::pin(connectivity(false))
    }

    #[cfg(feature = "selftest")]
    fn interface_cards(&self) -> Cards {
        Box::pin(std::future::ready(Ok(super::samples::cards())))
    }

    #[cfg(not(feature = "selftest"))]
    fn interface_cards(&self) -> Cards {
        Box::pin(toolbox_system::system_cards())
    }

    fn set_radio(&self, target: RadioTarget, blocked: bool) -> Result<(), ProbeError> {
        tracing::info!("dry run: {target:?} would be blocked: {blocked}");
        Ok(())
    }
}

impl fmt::Debug for NetworkSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetworkSetup").finish_non_exhaustive()
    }
}

impl Jobs {
    pub fn new(setup: NetworkSetup, ctx: egui::Context) -> Self {
        let (answers, live) = waking::channel(waking::repaint(ctx.clone()));
        Self {
            lane: Lane::new(setup.runtime.clone(), waking::repaint(ctx)),
            setup,
            answers,
            live,
        }
    }

    pub fn landed(&self) -> Vec<Landed> {
        let mut landed = self.lane.landed();
        landed.extend(self.live.try_iter());
        landed
    }

    /// Starts the live cards, once the page opened.
    pub(super) fn watch_interfaces(&self) {
        live::spawn(
            &self.setup.runtime,
            Arc::clone(&self.setup.actions),
            self.answers.clone(),
        );
    }

    /// NetworkManager answers over D-Bus, so this waits on the runtime.
    pub(super) fn read_interfaces(&self) {
        let cards = self.setup.actions.interface_cards();
        self.lane.future("interfaces", cards, |joined| {
            let read = joined.unwrap_or_else(|error| Err(CallError::Failed(error.to_string())));
            Landed::Interfaces(cards_or_none(read))
        });
    }

    /// A read that panicked leaves its parts failed.
    pub(super) fn read_status(&self) {
        let source = Arc::clone(&self.setup.source);
        self.lane.blocking(
            "status",
            move || StatusRead {
                route: Read::of("the default route", source.default_route()),
                radios: Read::of("the radios", source.radios()),
                access: source.radio_access(),
            },
            |status| {
                Landed::Status(status.unwrap_or(StatusRead {
                    route: Read::Failed,
                    radios: Read::Failed,
                    access: toolbox_core::features::network::RadioAccess::Missing,
                }))
            },
        );
    }

    pub(super) fn read_hardware(&self) {
        let source = Arc::clone(&self.setup.source);
        self.lane.blocking(
            "hardware",
            move || Hardware {
                pci: Read::of("lspci", source.pci_devices()),
                usb: Read::of("lsusb", source.usb_devices()),
            },
            |lists| {
                Landed::Hardware(lists.unwrap_or(Hardware {
                    pci: Read::Failed,
                    usb: Read::Failed,
                }))
            },
        );
    }

    pub(super) fn lookup_address(&self) {
        let address = Arc::clone(&self.setup.address);
        tracing::info!("asking for the public address");
        self.lane.blocking(
            "address",
            move || lookup_address(address.as_ref()),
            |answer| {
                Landed::Address(answer.unwrap_or_else(|stopped| {
                    Err(AddressProblem::NoAnswer(RequestError::NoAnswer(
                        stopped.to_string(),
                    )))
                }))
            },
        );
    }

    pub(super) fn set_radio(&self, target: RadioTarget, blocked: bool) {
        let actions = Arc::clone(&self.setup.actions);
        self.lane.blocking(
            "radio",
            move || actions.set_radio(target, blocked),
            |result| Landed::Radio(result.unwrap_or(Err(ProbeError::Panicked))),
        );
    }

    /// NetworkManager answers over D-Bus, so this waits on the runtime
    /// rather than a blocking thread.
    pub(super) fn check_connectivity(&self, recheck: bool) {
        let checked = self.setup.actions.connectivity(recheck);
        self.lane.future("connectivity", checked, |result| {
            // A check that died did not check.
            Landed::Connectivity(result.unwrap_or(Connectivity::Unknown))
        });
    }
}

/// The cards of a read, or none when NetworkManager does not run: the page
/// then hides the section. Other failures are logged.
pub(super) fn cards_or_none(
    read: Result<Vec<InterfaceCard>, CallError>,
) -> Option<Vec<InterfaceCard>> {
    match read {
        Ok(cards) => Some(cards),
        Err(error @ (CallError::Unavailable | CallError::Denied)) => {
            tracing::debug!("no interface cards: {error}");
            None
        }
        Err(error) => {
            tracing::warn!("the interface cards: {error}");
            None
        }
    }
}
