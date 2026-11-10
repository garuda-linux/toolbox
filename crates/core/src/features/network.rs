//! The Network page: what its status shows, the driver list, the device
//! lists and the plans its buttons start.
//!
//! Only blacklisting queues ([`crate::domains::ModuleBlacklist`]). Loading
//! and unloading a module, starting a service, ping and traceroute are
//! plans that run at once in the Console.

mod drivers;
mod status;
#[cfg(test)]
mod tests;
mod tools;

pub use drivers::{
    BlacklistEntry, Category, DriverFilter, DriverRow, DriverState, ModuleFacts, NetworkModule,
    Refusal, SwitchRefusal, driver_rows, loaded_modules, network_modules, plan_load, plan_switch,
    plan_unload, recheck_blacklist, set_blacklisted, visible,
};
pub use status::{
    Connectivity, DefaultRoute, GEOCLUE, Radio, RadioAccess, RadioKind, RadioSummary, RadioTarget,
    SERVICES, ServiceRow, flight_mode, plan_start, public_address, radio_summary, rfkill_arguments,
    service_rows,
};
pub use toolbox_state::network::hardware::{
    Device, DeviceKind, UsbInterface, copy_text, pci_devices, usb_devices,
};
pub use tools::{
    HOTSPOT_PACKAGE, HOTSPOT_PROGRAM, PackageState, TRACEROUTE_PACKAGE, package_state, plan_ping,
    plan_traceroute, toggle_install,
};
