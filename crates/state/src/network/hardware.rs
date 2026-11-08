//! The Hardware tab: the network devices `lspci -nn` and `lsusb` list,
//! with the interface classes of `/sys/bus/usb/devices`. All of it is
//! untrusted text: it is sorted into kinds here and shown as plain text.

const PCI_ETHERNET: &str = "0200";
/// Every other PCI network controller, in practice Wi-Fi cards.
const PCI_NETWORK: &str = "02";
const PCI_BLUETOOTH: &str = "0d11";
/// PCI's wireless controllers: 802.11a, 802.11b and "other".
const PCI_WIRELESS: [&str; 3] = ["0d20", "0d21", "0d80"];

const USB_WIFI_WORDS: [&str; 4] = ["wifi", "wi-fi", "wlan", "802.11"];
/// "Wireless" also names what talks to a receiver of its own. With one of
/// these words the device is no network adapter.
const USB_NOT_NETWORK: [&str; 9] = [
    "mouse",
    "keyboard",
    "receiver",
    "transceiver",
    "headset",
    "headphone",
    "audio",
    "controller",
    "gamepad",
];
const USB_ETHERNET_WORDS: [&str; 3] = ["ethernet", "rndis", "gigabit"];
/// Wireless controller, RF controller, Bluetooth programming interface.
const USB_BLUETOOTH: [u8; 3] = [0xe0, 0x01, 0x01];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeviceKind {
    Ethernet,
    Wireless,
    Bluetooth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UsbId {
    vendor: u16,
    product: u16,
}

/// One interface of a USB device as sysfs lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UsbInterface {
    device: UsbId,
    /// Class, subclass and protocol.
    class: [u8; 3],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    pub kind: DeviceKind,
    pub description: String,
    /// The whole line, which is what gets copied.
    pub raw: String,
}

impl UsbId {
    fn parse(vendor: &str, product: &str) -> Option<Self> {
        Some(Self {
            vendor: u16::from_str_radix(vendor.trim(), 16).ok()?,
            product: u16::from_str_radix(product.trim(), 16).ok()?,
        })
    }
}

impl UsbInterface {
    /// From the files `idVendor` and `idProduct` of the device and
    /// `bInterfaceClass`, `bInterfaceSubClass` and `bInterfaceProtocol` of
    /// the interface, each a hex number and a line break.
    pub fn from_sysfs(vendor: &str, product: &str, class: [&str; 3]) -> Option<Self> {
        let [class, subclass, protocol] = class.map(|value| u8::from_str_radix(value.trim(), 16));

        Some(Self {
            device: UsbId::parse(vendor, product)?,
            class: [class.ok()?, subclass.ok()?, protocol.ok()?],
        })
    }
}

/// The network devices in the output of `lspci -nn`, by their class code
/// (`Ethernet controller [0200]:`). The class text depends on the version
/// of the PCI id list, the code does not.
pub fn pci_devices(lspci: &str) -> Vec<Device> {
    lspci
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (class, description) = pci_class(line)?;
            let kind = if class == PCI_ETHERNET {
                DeviceKind::Ethernet
            } else if class == PCI_BLUETOOTH {
                DeviceKind::Bluetooth
            } else if class.starts_with(PCI_NETWORK) || PCI_WIRELESS.contains(&class) {
                DeviceKind::Wireless
            } else {
                return None;
            };

            Some(Device {
                kind,
                description: description.to_owned(),
                raw: line.to_owned(),
            })
        })
        .collect()
}

/// The network devices in the output of `lsusb`.
///
/// Plain `lsusb` prints no class, so a Bluetooth interface among
/// `interfaces` decides first, and then the words of the name. Wi-Fi and
/// Ethernet adapters mostly have interfaces of their vendor's own class,
/// which tells nothing.
pub fn usb_devices(lsusb: &str, interfaces: &[UsbInterface]) -> Vec<Device> {
    lsusb
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (id, description) = usb_description(line)?;
            let words = description.to_lowercase();
            let has = |list: &[&str]| list.iter().any(|word| words.contains(word));
            let bluetooth = interfaces
                .iter()
                .any(|interface| interface.device == id && interface.class == USB_BLUETOOTH);

            let kind = if bluetooth || words.contains("bluetooth") {
                DeviceKind::Bluetooth
            } else if has(&USB_WIFI_WORDS) || (words.contains("wireless") && !has(&USB_NOT_NETWORK))
            {
                DeviceKind::Wireless
            } else if has(&USB_ETHERNET_WORDS) {
                DeviceKind::Ethernet
            } else {
                return None;
            };

            Some(Device {
                kind,
                description: description.to_owned(),
                raw: line.to_owned(),
            })
        })
        .collect()
}

/// What "copy all" puts on the clipboard: the lines as the programs
/// printed them.
pub fn copy_text<'a>(devices: impl IntoIterator<Item = &'a Device>) -> String {
    devices
        .into_iter()
        .map(|device| device.raw.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn pci_class(line: &str) -> Option<(&str, &str)> {
    let (before, description) = line.split_once("]: ")?;
    let (_, class) = before.rsplit_once('[')?;
    let is_class = class.len() == 4 && class.chars().all(|digit| digit.is_ascii_hexdigit());

    is_class.then_some((class, description.trim()))
}

/// `8087:0029` and `Intel Corp. AX200 Bluetooth` for `Bus 001 Device 004:
/// ID 8087:0029 Intel Corp. AX200 Bluetooth`.
fn usb_description(line: &str) -> Option<(UsbId, &str)> {
    let (_, rest) = line.split_once(": ID ")?;
    let (id, description) = rest.split_once(' ')?;
    let (vendor, product) = id.split_once(':')?;
    let description = description.trim();

    (!description.is_empty()).then_some((UsbId::parse(vendor, product)?, description))
}
