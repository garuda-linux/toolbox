//! What the Network page asks outside the snapshot: NetworkManager's
//! connectivity over D-Bus and, only after a click, the public address
//! from one HTTPS request. What it does outside a plan is in `actions`.

#[cfg(test)]
mod tests;

use std::net::IpAddr;
use std::time::Duration;

use toolbox_https::{Https, RequestError, Rules};
use toolbox_plan::UnitScope;
use toolbox_state::network::{ADDRESS_SERVICE, Connectivity, public_address};
use zbus::OwnedVariant;

use super::network_manager::{
    CHECK_CONNECTIVITY, CONNECTIVITY_PROPERTY, connectivity_from_value, connectivity_property_body,
};
use crate::bus::{CallError, call, connect};

/// NetworkManager's own check waits for its test page. The page shows
/// "unknown" rather than wait longer.
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);

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

/// What NetworkManager says about the way to the internet, unknown
/// without it. With `recheck` it checks now, which makes NetworkManager
/// fetch its test page. Without, the state of its last check is read.
pub async fn connectivity(recheck: bool) -> Connectivity {
    let connection = match connect(UnitScope::System).await {
        Ok(connection) => connection,
        Err(error) => {
            tracing::debug!("no connectivity state: {error}");
            return Connectivity::Unknown;
        }
    };

    if recheck {
        let check = call::<_, u32>(&connection, &CHECK_CONNECTIVITY, &());

        match tokio::time::timeout(CHECK_TIMEOUT, check).await {
            Ok(Ok(code)) => return Connectivity::from_code(code),
            Ok(Err(CallError::Unavailable)) => return Connectivity::Unknown,
            // A session that may not ask for a check may still read.
            Ok(Err(error)) => tracing::debug!("NetworkManager did not check: {error}"),
            Err(_) => tracing::debug!("NetworkManager's check took too long"),
        }
    }

    let body = connectivity_property_body();

    match call::<_, OwnedVariant>(&connection, &CONNECTIVITY_PROPERTY, &body).await {
        Ok(value) => connectivity_from_value(&value),
        Err(error) => {
            tracing::debug!("no connectivity state: {error}");
            Connectivity::Unknown
        }
    }
}
