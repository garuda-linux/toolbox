//! What the Network page keeps between frames: the route, the radios, the
//! internet as NetworkManager sees it, the device lists and the public
//! address once asked.
//!
//! Everything is read by the page itself, once it opened, again on
//! Refresh and after every plan. The reads block or wait for D-Bus, so
//! they run on the runtime and wake the frame with their result.

mod jobs;
#[cfg(test)]
mod tests;

use std::net::IpAddr;

pub use jobs::{DryRunActions, DryRunAddress, NetworkSetup};
use jobs::{Jobs, Landed};
use toolbox_core::features::network::{
    Connectivity, DefaultRoute, Device, Radio, RadioAccess, RadioTarget,
};
use toolbox_services::address::AddressProblem;
use toolbox_system::ProbeError;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Read<T> {
    #[default]
    Loading,
    /// The program or file it needs is not on this system.
    Unavailable,
    Failed,
    Known(T),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusRead {
    pub route: Read<DefaultRoute>,
    pub radios: Read<Vec<Radio>>,
    pub access: RadioAccess,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hardware {
    pub pci: Read<Vec<Device>>,
    pub usb: Read<Vec<Device>>,
}

/// The public address, asked only after a click.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Address {
    #[default]
    NotAsked,
    Loading,
    Known(IpAddr),
    Failed,
}

#[derive(Debug, Default)]
pub struct Network {
    opened: bool,
    status: Option<StatusRead>,
    connectivity: Option<Connectivity>,
    hardware: Option<Hardware>,
    address: Address,
    /// The last radio switch failed. A later one that works clears it.
    radio_failed: bool,
    /// `None` in tests that need no reads: nothing is read.
    jobs: Option<Jobs>,
}

impl<T> Read<T> {
    /// `result` as a read. A failure is logged here, the page only says
    /// that it failed.
    pub fn of(what: &str, result: Result<T, ProbeError>) -> Self {
        match result {
            Ok(value) => Self::Known(value),
            Err(ProbeError::Unavailable) => Self::Unavailable,
            Err(error) => {
                tracing::warn!("{what}: {}", error.reason().unwrap_or_default());
                Self::Failed
            }
        }
    }

    pub const fn known(&self) -> Option<&T> {
        match self {
            Self::Known(value) => Some(value),
            Self::Loading | Self::Unavailable | Self::Failed => None,
        }
    }
}

impl Network {
    pub fn new(setup: Option<NetworkSetup>, ctx: egui::Context) -> Self {
        Self {
            jobs: setup.map(|setup| Jobs::new(setup, ctx)),
            ..Self::default()
        }
    }

    pub const fn status(&self) -> Option<&StatusRead> {
        self.status.as_ref()
    }

    pub const fn connectivity(&self) -> Option<Connectivity> {
        self.connectivity
    }

    pub const fn hardware(&self) -> Option<&Hardware> {
        self.hardware.as_ref()
    }

    pub const fn address(&self) -> &Address {
        &self.address
    }

    pub const fn radio_failed(&self) -> bool {
        self.radio_failed
    }

    /// The page shows: its first look reads everything and asks
    /// NetworkManager to check the internet now.
    pub fn open(&mut self) {
        if !self.opened {
            self.opened = true;
            self.read_all(true);
        }
    }

    pub fn refresh(&mut self) {
        self.read_all(true);
    }

    /// A plan may have changed drivers, services or packages. Read again,
    /// once the page was opened, without a fresh check.
    pub fn plan_finished(&mut self) {
        if self.opened {
            self.read_all(false);
        }
    }

    pub fn set_radio(&self, target: RadioTarget, blocked: bool) {
        if let Some(jobs) = &self.jobs {
            jobs.set_radio(target, blocked);
        }
    }

    pub fn lookup_address(&mut self) {
        let Some(jobs) = &self.jobs else {
            return;
        };

        if self.address != Address::Loading {
            self.address = Address::Loading;
            jobs.lookup_address();
        }
    }

    /// Forgets the address, so it shows only while the user wants it.
    pub fn forget_address(&mut self) {
        if self.address != Address::Loading {
            self.address = Address::NotAsked;
        }
    }

    pub fn take(&mut self) {
        let landed: Vec<Landed> = self.jobs.as_ref().map(Jobs::landed).unwrap_or_default();

        for result in landed {
            match result {
                Landed::Status(status) => self.status = Some(status),
                Landed::Connectivity(state) => self.connectivity = Some(state),
                Landed::Hardware(lists) => self.hardware = Some(lists),
                Landed::Address(answer) => self.address = settled(answer),
                Landed::Radio(result) => {
                    self.radio_failed = result.is_err();
                    if let Some(jobs) = &self.jobs {
                        jobs.read_status();
                    }
                }
            }
        }
    }

    fn read_all(&self, recheck: bool) {
        if let Some(jobs) = &self.jobs {
            jobs.read_status();
            jobs.check_connectivity(recheck);
            jobs.read_hardware();
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl Network {
    /// The state a page test shows, without reads.
    pub fn showing(
        status: Option<StatusRead>,
        connectivity: Option<Connectivity>,
        hardware: Option<Hardware>,
        address: Address,
    ) -> Self {
        Self {
            opened: true,
            status,
            connectivity,
            hardware,
            address,
            ..Self::default()
        }
    }
}

/// Only the address is kept: of a failure nothing but that it failed, as
/// its reason may hold parts of the answer.
fn settled(answer: Result<IpAddr, AddressProblem>) -> Address {
    match answer {
        Ok(address) => Address::Known(address),
        Err(problem) => {
            tracing::info!("no public address: {problem:?}");
            Address::Failed
        }
    }
}
