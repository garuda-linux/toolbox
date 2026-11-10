use std::fs;
use std::path::Path;

use toolbox_system::testing::network::BARE_METAL_SYS_BUS_USB_DEVICES;

use crate::features::network::{DeviceKind, UsbInterface, copy_text, pci_devices, usb_devices};

/// A bare-metal laptop: a Realtek Ethernet port and a `MediaTek` MT7922 Wi-Fi
/// card, the MT7922's Bluetooth on USB as a Foxconn device.
const LAPTOP_LSPCI: &str = toolbox_system::testing::network::BARE_METAL_LSPCI_NN;
const LAPTOP_LSUSB: &str = toolbox_system::testing::network::BARE_METAL_LSUSB;
/// The Lima VM, with one virtio network device and no USB device but
/// the root hubs.
const LIMA_LSPCI: &str = toolbox_system::testing::network::LSPCI_NN;
const LIMA_LSUSB: &str = toolbox_system::testing::network::LSUSB_STDOUT;

/// A file of the laptop's `/sys/bus/usb/devices` capture.
fn sysfs(file: &str) -> String {
    fs::read_to_string(Path::new(BARE_METAL_SYS_BUS_USB_DEVICES).join(file)).unwrap()
}

/// The first interface of the captured USB device `device`.
fn first_interface(device: &str) -> UsbInterface {
    let class = [
        sysfs(&format!("{device}:1.0/bInterfaceClass")),
        sysfs(&format!("{device}:1.0/bInterfaceSubClass")),
        sysfs(&format!("{device}:1.0/bInterfaceProtocol")),
    ];

    UsbInterface::from_sysfs(
        &sysfs(&format!("{device}/idVendor")),
        &sysfs(&format!("{device}/idProduct")),
        class.each_ref().map(String::as_str),
    )
    .unwrap()
}

/// The first interface of the Foxconn device `3-2`.
fn laptop_bluetooth() -> UsbInterface {
    first_interface("3-2")
}

/// The first interface of the RTL8153 `2-1.4.4`, vendor specific.
fn laptop_ethernet() -> UsbInterface {
    first_interface("2-1.4.4")
}

fn line_of<'text>(listing: &'text str, start: &str) -> &'text str {
    listing
        .lines()
        .find(|line| line.starts_with(start))
        .unwrap_or_else(|| panic!("{start} is listed"))
}

#[test]
fn pci_devices_are_kinded_by_class_code() {
    let found: Vec<(DeviceKind, String)> = pci_devices(LAPTOP_LSPCI)
        .into_iter()
        .map(|device| (device.kind, device.description))
        .collect();

    assert_eq!(
        found,
        [
            (
                DeviceKind::Ethernet,
                "Realtek Semiconductor Co., Ltd. RTL8111/8168/8211/8411 PCI Express Gigabit \
                 Ethernet Controller [10ec:8168] (rev 15)"
                    .to_owned()
            ),
            (
                DeviceKind::Wireless,
                "MEDIATEK Corp. MT7922 802.11ax PCI Express Wireless Network Adapter [14c3:0616]"
                    .to_owned()
            ),
        ]
    );

    let lima: Vec<_> = pci_devices(LIMA_LSPCI)
        .into_iter()
        .map(|device| (device.kind, device.description))
        .collect();

    assert_eq!(
        lima,
        [(
            DeviceKind::Ethernet,
            "Red Hat, Inc. Virtio network device [1af4:1000]".to_owned()
        )]
    );
}

#[test]
fn a_bluetooth_controller_on_pci_is_bluetooth() {
    // The MT7922 line edited to the class code of a Bluetooth controller.
    let bluetooth =
        line_of(LAPTOP_LSPCI, "04:00.0").replace("Network controller [0280]", "Bluetooth [0d11]");

    let kinds: Vec<_> = pci_devices(&bluetooth)
        .into_iter()
        .map(|device| device.kind)
        .collect();

    assert_eq!(kinds, [DeviceKind::Bluetooth]);
}

/// The Foxconn device is the MT7922's Bluetooth, which only its
/// interface class tells.
#[test]
fn a_bluetooth_interface_makes_a_usb_device_bluetooth() {
    let found: Vec<(DeviceKind, String)> =
        usb_devices(LAPTOP_LSUSB, &[laptop_ethernet(), laptop_bluetooth()])
            .into_iter()
            .map(|device| (device.kind, device.description))
            .collect();

    assert_eq!(
        found,
        [
            (
                DeviceKind::Ethernet,
                "Realtek Semiconductor Corp. RTL8153 Gigabit Ethernet Adapter".to_owned()
            ),
            (
                DeviceKind::Bluetooth,
                "Foxconn / Hon Hai Wireless_Device".to_owned()
            ),
        ]
    );
    assert_eq!(usb_devices(LIMA_LSUSB, &[]), []);
}

/// Without the interfaces of a device only the words of its name count.
#[test]
fn without_its_interfaces_a_usb_device_is_kinded_by_its_name() {
    let kinds: Vec<DeviceKind> = usb_devices(LAPTOP_LSUSB, &[])
        .into_iter()
        .map(|device| device.kind)
        .collect();

    assert_eq!(kinds, [DeviceKind::Ethernet, DeviceKind::Wireless]);
}

#[test]
fn sysfs_values_that_are_no_hex_numbers_are_refused() {
    let vendor = sysfs("3-2/idVendor");
    let product = sysfs("3-2/idProduct");
    let class = sysfs("3-2:1.0/bInterfaceClass");
    let rest = [
        sysfs("3-2:1.0/bInterfaceSubClass"),
        sysfs("3-2:1.0/bInterfaceProtocol"),
    ];

    // Edit: the vendor of the capture emptied, then its class made `zz`.
    assert!(UsbInterface::from_sysfs("", &product, [&class, &rest[0], &rest[1]]).is_none());
    assert!(UsbInterface::from_sysfs(&vendor, &product, ["zz\n", &rest[0], &rest[1]]).is_none());
}

#[test]
fn a_wireless_mouse_is_no_network_device() {
    // The laptop's ROCCAT mouse, edited to call itself wireless.
    let mouse =
        line_of(LAPTOP_LSUSB, "Bus 001 Device 006").replace("Kone XP", "Kone XP Wireless Mouse");

    assert_eq!(usb_devices(&mouse, &[]), []);
}

#[test]
fn the_whole_lines_are_what_gets_copied() {
    let devices = pci_devices(LAPTOP_LSPCI);

    assert_eq!(
        copy_text(&devices),
        format!(
            "{}\n{}",
            line_of(LAPTOP_LSPCI, "03:00.0"),
            line_of(LAPTOP_LSPCI, "04:00.0")
        )
    );
    assert_eq!(copy_text(&[]), "");
}

#[test]
fn lines_that_are_no_device_lines_are_left_out() {
    let ethernet = line_of(LAPTOP_LSPCI, "03:00.0");
    let adapter = line_of(LAPTOP_LSUSB, "Bus 002 Device 006");

    assert_eq!(pci_devices(""), []);

    // The class code cut from one line, and the name from the other.
    assert_eq!(pci_devices(&ethernet.replace(" [0200]", "")), []);
    assert_eq!(usb_devices("", &[]), []);
    assert_eq!(usb_devices(adapter.get(..33).unwrap(), &[]), []);
}
