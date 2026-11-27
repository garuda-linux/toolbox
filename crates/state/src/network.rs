mod address;
pub mod drivers;
pub mod hardware;

use std::fmt;
use std::net::IpAddr;

pub use address::{ADDRESS_SERVICE, public_address};
pub use drivers::{
    BlacklistEntry, Category, ModuleFacts, NetworkModule, loaded_modules, network_modules,
};
pub use hardware::{Device, DeviceKind, UsbInterface, copy_text, pci_devices, usb_devices};
use toolbox_text::OutsideText;

/// What NetworkManager says about the way to the internet
/// (`NMConnectivityState`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connectivity {
    /// NetworkManager does not run or did not check.
    Unknown,
    None,
    /// A login page of a hotspot stands in the way.
    Portal,
    /// A network, but no way to the internet.
    Limited,
    Full,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DefaultRoute {
    pub gateway: Option<IpAddr>,
    /// A kernel name, shown as plain text.
    pub interface: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioKind {
    Wifi,
    Bluetooth,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Radio {
    /// `None` for kinds the page does not show (NFC, WWAN, …).
    pub kind: Option<RadioKind>,
    /// Blocked by software, which `rfkill` can undo.
    pub soft: bool,
    /// Blocked by a hardware switch, which only that switch undoes.
    pub hard: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RadioSummary {
    pub present: bool,
    /// Every radio of the kind is blocked.
    pub blocked: bool,
    /// A hardware switch blocks one of them.
    pub hard: bool,
}

/// One interface of the Network page: an activated Ethernet or Wi-Fi
/// device with the connection it carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceCard {
    /// The active connection's name, the card's title. A name the user or
    /// a network chose, so it is outside text.
    pub connection: OutsideText,
    /// A kernel name. The kernel allows most bytes in it, so it is outside
    /// text too.
    pub interface: OutsideText,
    /// `None` when NetworkManager gives no hardware address for the device.
    pub mac: Option<String>,
    pub ipv4: Vec<Subnet>,
    /// Global and unique local addresses first, link-local last.
    pub ipv6: Vec<Subnet>,
    /// The servers NetworkManager holds for this interface, IPv4 and IPv6
    /// together.
    pub dns: Vec<IpAddr>,
    pub link: Link,
}

/// An address with its prefix length, shown as `address/prefix`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Subnet {
    pub address: IpAddr,
    pub prefix: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Link {
    Wired {
        /// Megabits per second, `None` while the link reports none.
        speed: Option<u32>,
    },
    Wireless {
        /// `None` when the device has no active access point.
        ssid: Option<OutsideText>,
        signal: Option<Signal>,
    },
}

/// The strength of the active access point, in percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signal {
    pub percent: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalQuality {
    Excellent,
    Good,
    Fair,
    Weak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioAccess {
    Allowed,
    /// `/dev/rfkill` exists but the session may not write to it.
    Denied,
    /// No `/dev/rfkill`: a kernel without rfkill, or a container.
    Missing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioTarget {
    Wifi,
    Bluetooth,
    /// Flight mode.
    All,
}

impl Connectivity {
    /// Codes NetworkManager does not document count as unknown.
    pub const fn from_code(code: u32) -> Self {
        match code {
            1 => Self::None,
            2 => Self::Portal,
            3 => Self::Limited,
            4 => Self::Full,
            _ => Self::Unknown,
        }
    }
}

impl Signal {
    /// nmcli's bar thresholds: Excellent over 80, Good over 55, Fair over
    /// 30, Weak otherwise.
    pub const fn quality(self) -> SignalQuality {
        match self.percent {
            81.. => SignalQuality::Excellent,
            56..=80 => SignalQuality::Good,
            31..=55 => SignalQuality::Fair,
            0..=30 => SignalQuality::Weak,
        }
    }
}

impl fmt::Display for Subnet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.prefix)
    }
}

impl RadioKind {
    /// The kernel's name of the kind in `/sys/class/rfkill/*/type`.
    pub fn from_sysfs(text: &str) -> Option<Self> {
        match text.trim() {
            "wlan" => Some(Self::Wifi),
            "bluetooth" => Some(Self::Bluetooth),
            _ => None,
        }
    }
}

impl RadioTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wifi => "wifi",
            Self::Bluetooth => "bluetooth",
            Self::All => "all",
        }
    }
}

/// What `rfkill` gets after its name: two fixed words, never text from
/// the page.
pub const fn rfkill_arguments(target: RadioTarget, blocked: bool) -> [&'static str; 2] {
    [if blocked { "block" } else { "unblock" }, target.as_str()]
}
