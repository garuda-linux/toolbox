//! The public address the Network page shows after a click: one HTTPS
//! request to KDE's service.

#[cfg(test)]
mod tests;

use std::net::IpAddr;
use std::time::Duration;

use toolbox_https::{Https, RequestError, Rules};
use toolbox_state::network::{ADDRESS_SERVICE, public_address};

/// A lookup answers a few lines. Whoever clicked waits for it.
const ADDRESS_TIMEOUT: Duration = Duration::from_secs(20);

const ADDRESS_HOST: &str = "geoip.kde.org";

/// The service's answer is one short JSON object.
const MAX_ADDRESS_BYTES: u64 = 64 << 10;

/// Blocks, so call it off the GUI thread.
pub trait AddressSource: Send + Sync {
    /// # Errors
    ///
    /// Why there is no answer: no network, a TLS problem, a timeout.
    fn answer(&self) -> Result<String, RequestError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddressProblem {
    NoAnswer(RequestError),
    NotUnderstood,
}

/// [`AddressSource`] through KDE's service, the one request the page
/// ever sends.
pub struct KdeGeoIp(Https);

impl KdeGeoIp {
    #[must_use = "the lookup does nothing until asked"]
    pub fn new() -> Self {
        Self(Https::new(&Rules {
            host: ADDRESS_HOST,
            timeout: ADDRESS_TIMEOUT,
            max_body_bytes: MAX_ADDRESS_BYTES,
            // ureq's own: the service was never told the program.
            user_agent: None,
        }))
    }
}

impl Default for KdeGeoIp {
    fn default() -> Self {
        Self::new()
    }
}

impl AddressSource for KdeGeoIp {
    fn answer(&self) -> Result<String, RequestError> {
        let response = self.0.get(ADDRESS_SERVICE, &[])?;
        self.0.text(response)
    }
}

/// The public address. The answer is dropped right here: of all it holds
/// (country, city, position) only the address goes on, and nothing of it
/// is logged or stored.
///
/// # Errors
///
/// [`AddressProblem`] without an answer or without an address in it.
pub fn lookup_address(source: &dyn AddressSource) -> Result<IpAddr, AddressProblem> {
    let answer = source.answer().map_err(AddressProblem::NoAnswer)?;

    public_address(&answer).ok_or(AddressProblem::NotUnderstood)
}
