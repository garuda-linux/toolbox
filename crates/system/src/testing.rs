//! Captures other crates' tests read, by the module path that parses them,
//! and the fakes of the ports, which answer without touching the system.

pub mod aur_orders;
pub mod aur_updates;
pub mod file_chooser;
pub mod host;
pub mod launch;
pub mod locales;
pub mod look_portal;
pub mod network;
pub mod notify;
pub mod packages;
pub mod status;
pub mod systemd;
pub mod unit_signals;
