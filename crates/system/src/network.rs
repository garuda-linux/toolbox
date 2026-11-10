//! What the Network page reads: the default route, the radio switches and
//! the device lists, plus the one program it runs outside the Console
//! (`rfkill`, as the user). Kernel modules are in [`modules`], the
//! messages for NetworkManager in [`connectivity`].

mod actions;
mod modules;
mod network_manager;
mod online;
#[cfg(test)]
mod tests;
mod usb;

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io;
use std::net::IpAddr;
use std::path::PathBuf;

pub use actions::{Checked, NetworkActions, SystemActions};
pub use modules::{ModuleSource, SystemModules};
pub use online::{AddressProblem, AddressSource, KdeGeoIp, connectivity, lookup_address};
use serde::Deserialize;
use toolbox_state::network::{
    DefaultRoute, Device, Radio, RadioAccess, RadioKind, RadioTarget, pci_devices,
    rfkill_arguments, usb_devices,
};

use crate::error::{ProbeError, from_io};
use crate::{command, files};

const IP: &str = "/usr/bin/ip";
const RFKILL: &str = "/usr/bin/rfkill";
const LSPCI: &str = "/usr/bin/lspci";
const LSUSB: &str = "/usr/bin/lsusb";

const RFKILL_CLASS: &str = "sys/class/rfkill";
const RFKILL_DEVICE: &str = "dev/rfkill";

/// The kernel's limit for an interface name (`IFNAMSIZ` without the NUL).
const MAX_INTERFACE_CHARS: usize = 15;

pub trait NetworkSource: Send + Sync {
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `ip`, [`ProbeError::Failed`]
    /// when it prints nothing usable.
    fn default_route(&self) -> Result<DefaultRoute, ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Failed`] when the switches cannot be listed.
    fn radios(&self) -> Result<Vec<Radio>, ProbeError>;

    fn radio_access(&self) -> RadioAccess;

    /// Blocks or unblocks radios with `rfkill`, as the user.
    ///
    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `rfkill`,
    /// [`ProbeError::Failed`] with its first complaint when it refuses.
    fn set_radio(&self, target: RadioTarget, blocked: bool) -> Result<(), ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `lspci`,
    /// [`ProbeError::Failed`] when it fails.
    fn pci_devices(&self) -> Result<Vec<Device>, ProbeError>;

    /// # Errors
    ///
    /// [`ProbeError::Unavailable`] without `lsusb`,
    /// [`ProbeError::Failed`] when it fails.
    fn usb_devices(&self) -> Result<Vec<Device>, ProbeError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkPrograms {
    pub ip: OsString,
    pub rfkill: OsString,
    pub lspci: OsString,
    pub lsusb: OsString,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemNetwork {
    root: PathBuf,
    programs: NetworkPrograms,
}

#[derive(Deserialize)]
struct Route {
    #[serde(default)]
    gateway: Option<String>,
    #[serde(default)]
    dev: Option<String>,
}

impl Default for NetworkPrograms {
    fn default() -> Self {
        Self {
            ip: IP.into(),
            rfkill: RFKILL.into(),
            lspci: LSPCI.into(),
            lsusb: LSUSB.into(),
        }
    }
}

impl SystemNetwork {
    #[must_use = "the adapter does nothing until asked"]
    pub fn system() -> Self {
        Self::new("/", NetworkPrograms::default())
    }

    #[must_use = "the adapter does nothing until asked"]
    pub(crate) fn new(root: impl Into<PathBuf>, programs: NetworkPrograms) -> Self {
        Self {
            root: root.into(),
            programs,
        }
    }
}

impl NetworkSource for SystemNetwork {
    fn default_route(&self) -> Result<DefaultRoute, ProbeError> {
        let output = command::output(&self.programs.ip, &["-json", "route", "show", "default"])?;

        parse_default_route(&output)
    }

    fn radios(&self) -> Result<Vec<Radio>, ProbeError> {
        let folder = self.root.join(RFKILL_CLASS);
        let entries = match std::fs::read_dir(&folder) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(from_io(folder.display(), error)),
        };

        let mut switches: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();

        switches.sort();

        Ok(switches
            .iter()
            .filter_map(|switch| {
                let read = |name: &str| files::read(&switch.join(name)).ok();

                Some(Radio {
                    kind: RadioKind::from_sysfs(&read("type")?),
                    soft: read("soft")?.trim() == "1",
                    hard: read("hard")?.trim() == "1",
                })
            })
            .collect())
    }

    /// Opening the device for writing is the question itself: rfkill does
    /// the same, and nothing is written.
    fn radio_access(&self) -> RadioAccess {
        match OpenOptions::new()
            .write(true)
            .open(self.root.join(RFKILL_DEVICE))
        {
            Ok(_) => RadioAccess::Allowed,
            Err(error) if error.kind() == io::ErrorKind::NotFound => RadioAccess::Missing,
            Err(error) => {
                tracing::debug!("no access to /{RFKILL_DEVICE}: {error}");
                RadioAccess::Denied
            }
        }
    }

    fn set_radio(&self, target: RadioTarget, blocked: bool) -> Result<(), ProbeError> {
        command::output(&self.programs.rfkill, &rfkill_arguments(target, blocked)).map(drop)
    }

    fn pci_devices(&self) -> Result<Vec<Device>, ProbeError> {
        command::output(&self.programs.lspci, &["-nn"]).map(|output| pci_devices(&output))
    }

    fn usb_devices(&self) -> Result<Vec<Device>, ProbeError> {
        command::output(&self.programs.lsusb, &[])
            .map(|output| usb_devices(&output, &usb::interfaces(&self.root)))
    }
}

/// The first route, as `ip route` prints the preferred one first.
fn parse_default_route(json: &str) -> Result<DefaultRoute, ProbeError> {
    let routes: Vec<Route> = serde_json::from_str(json.trim())
        .map_err(|error| ProbeError::Failed(format!("ip printed no route list: {error}")))?;

    Ok(routes
        .into_iter()
        .next()
        .map_or_else(DefaultRoute::default, |route| DefaultRoute {
            gateway: route
                .gateway
                .and_then(|gateway| gateway.parse::<IpAddr>().ok()),
            interface: route.dev.filter(|name| is_interface_name(name)),
        }))
}

fn is_interface_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().count() <= MAX_INTERFACE_CHARS
        && name
            .chars()
            .all(|character| character.is_ascii_graphic() && character != '/')
}
