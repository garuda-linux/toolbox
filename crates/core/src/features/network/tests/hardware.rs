use crate::features::network::{DeviceKind, copy_text, pci_devices, usb_devices};

const LSPCI: &str = "\
00:00.0 Host bridge [0600]: Intel Corporation Device [8086:a706] (rev 01)
00:14.3 Network controller [0280]: Intel Corporation Raptor Lake PCH CNVi WiFi [8086:7a70] (rev 11)
02:00.0 Ethernet controller [0200]: Realtek Semiconductor Co., Ltd. RTL8125 2.5GbE Controller [10ec:8125] (rev 05)
03:00.0 Wireless controller [0d11]: Some Vendor Bluetooth Module [1234:5678]
04:00.0 Non-Volatile memory controller [0108]: Samsung Electronics Co Ltd NVMe SSD Controller [144d:a80a]
05:00.0 Audio device [0403]: Ethernet controller fan club [dead:beef]
";

const LSUSB: &str = "\
Bus 001 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub
Bus 001 Device 004: ID 8087:0033 Intel Corp. AX211 Bluetooth
Bus 001 Device 005: ID 0bda:8153 Realtek Semiconductor Corp. RTL8153 Gigabit Ethernet Adapter
Bus 001 Device 006: ID 0e8d:7961 MediaTek Inc. Wireless_Device
Bus 001 Device 007: ID 2357:0138 TP-Link 802.11ac NIC
Bus 001 Device 008: ID 046d:c52b Logitech, Inc. Unifying Receiver
Bus 001 Device 009: ID 045e:0745 Microsoft Corp. Nano Transceiver v1.0 for Wireless Mouse
Bus 001 Device 010: ID 148f:5370 Ralink Technology, Corp. RT5370 Wireless Adapter
Bus 001 Device 011: ID 18d1:4ee4 Google Inc. Nexus/Pixel Device (RNDIS)
";

#[test]
fn pci_devices_are_sorted_by_class_code() {
    let devices = pci_devices(LSPCI);
    let found: Vec<(DeviceKind, &str)> = devices
        .iter()
        .map(|device| (device.kind, device.description.as_str()))
        .collect();

    assert_eq!(
        found,
        [
            (
                DeviceKind::Wireless,
                "Intel Corporation Raptor Lake PCH CNVi WiFi [8086:7a70] (rev 11)"
            ),
            (
                DeviceKind::Ethernet,
                "Realtek Semiconductor Co., Ltd. RTL8125 2.5GbE Controller [10ec:8125] (rev 05)"
            ),
            (
                DeviceKind::Bluetooth,
                "Some Vendor Bluetooth Module [1234:5678]"
            ),
        ]
    );
}

#[test]
fn usb_devices_are_sorted_by_the_words_of_their_names() {
    let found: Vec<(DeviceKind, String)> = usb_devices(LSUSB)
        .into_iter()
        .map(|device| (device.kind, device.description))
        .collect();

    assert_eq!(
        found,
        [
            (
                DeviceKind::Bluetooth,
                "Intel Corp. AX211 Bluetooth".to_owned()
            ),
            (
                DeviceKind::Ethernet,
                "Realtek Semiconductor Corp. RTL8153 Gigabit Ethernet Adapter".to_owned()
            ),
            (
                DeviceKind::Wireless,
                "MediaTek Inc. Wireless_Device".to_owned()
            ),
            (DeviceKind::Wireless, "TP-Link 802.11ac NIC".to_owned()),
            (
                DeviceKind::Wireless,
                "Ralink Technology, Corp. RT5370 Wireless Adapter".to_owned()
            ),
            (
                DeviceKind::Ethernet,
                "Google Inc. Nexus/Pixel Device (RNDIS)".to_owned()
            ),
        ]
    );
}

#[test]
fn a_wireless_mouse_is_no_network_device() {
    let mouse = "Bus 001 Device 009: ID 045e:0745 Microsoft Corp. Wireless Mouse\n";

    assert_eq!(usb_devices(mouse), []);
}

#[test]
fn the_whole_lines_are_what_gets_copied() {
    let devices = pci_devices(LSPCI);

    assert_eq!(
        copy_text(&devices[..2]),
        "00:14.3 Network controller [0280]: Intel Corporation Raptor Lake PCH CNVi WiFi \
         [8086:7a70] (rev 11)\n02:00.0 Ethernet controller [0200]: Realtek Semiconductor Co., \
         Ltd. RTL8125 2.5GbE Controller [10ec:8125] (rev 05)"
    );
    assert_eq!(copy_text(&[]), "");
}

#[test]
fn lines_that_are_no_device_lines_are_left_out() {
    assert_eq!(pci_devices("pcilib: Cannot open /proc/bus/pci\n\n"), []);
    assert_eq!(
        pci_devices("02:00.0 Ethernet controller: no class code"),
        []
    );
    assert_eq!(usb_devices("unable to initialize usb spec\n"), []);
    assert_eq!(usb_devices("Bus 001 Device 002: ID 0bda:8153 "), []);
}
