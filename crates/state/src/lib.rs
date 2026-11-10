//! The system as the probes see it: plain state for the snapshot.
//!
//! Everything here is data with validation and no I/O, so the probes
//! build it without knowing queued changes.

pub mod accounts;
pub mod aur;
pub mod boot;
pub mod bootrepair;
pub mod configs;
pub mod diagnostics;
pub mod kernels;
pub mod locales;
pub mod network;
pub mod packages;
pub mod parts;
pub mod probes;
pub mod services;
pub mod settings;
pub mod snapshot;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod toggle;

pub use snapshot::Snapshot;
pub use toggle::Toggle;
