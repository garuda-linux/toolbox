pub mod modules;

pub const BARE_METAL_LSPCI_NN: &str = include_str!("../network/fixtures/bare-metal/lspci-nn");
pub const BARE_METAL_LSUSB: &str = include_str!("../network/fixtures/bare-metal/lsusb");
pub const LSPCI_NN: &str = include_str!("../network/fixtures/lspci-nn");
pub const LSUSB_STDOUT: &str = include_str!("../network/fixtures/lsusb.stdout");

/// The folder of the laptop's `/sys/bus/usb/devices` capture, read file by file.
pub const BARE_METAL_SYS_BUS_USB_DEVICES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/network/fixtures/bare-metal/sys-bus-usb-devices"
);
