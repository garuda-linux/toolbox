//! The interfaces of the USB devices in `/sys/bus/usb/devices`, where an
//! interface is the entry `<device>:<configuration>.<interface>` next to
//! the entry of its device.

use std::path::Path;

use toolbox_state::network::UsbInterface;

use crate::files;

const USB_DEVICES: &str = "sys/bus/usb/devices";

/// Empty when the folder cannot be read: the devices are then kinded by
/// their names alone. An entry with a file missing is left out.
pub fn interfaces(root: &Path) -> Vec<UsbInterface> {
    let folder = root.join(USB_DEVICES);
    let Ok(entries) = std::fs::read_dir(&folder) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let interface = name.to_str()?;
            let (device, _) = interface.split_once(':')?;
            let read = |item: &str, file: &str| files::read(&folder.join(item).join(file)).ok();

            UsbInterface::from_sysfs(
                &read(device, "idVendor")?,
                &read(device, "idProduct")?,
                [
                    &read(interface, "bInterfaceClass")?,
                    &read(interface, "bInterfaceSubClass")?,
                    &read(interface, "bInterfaceProtocol")?,
                ],
            )
        })
        .collect()
}
