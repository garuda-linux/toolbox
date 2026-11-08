//! The public address, asked only after a click: the one service and the
//! one part of its answer the page reads.

use std::net::IpAddr;

/// KDE's service, as Plasma's own setup uses it.
pub const ADDRESS_SERVICE: &str = "https://geoip.kde.org/v1/ubiquity";

const ADDRESS_OPEN: &str = "<Ip>";
const ADDRESS_CLOSE: &str = "</Ip>";

/// The address in the service's answer. Only the `Ip` element is read,
/// and only when it holds an IP address: nothing else of the answer can
/// reach the page.
pub fn public_address(answer: &str) -> Option<IpAddr> {
    let (_, rest) = answer.split_once(ADDRESS_OPEN)?;
    let (address, _) = rest.split_once(ADDRESS_CLOSE)?;

    address.trim().parse().ok()
}
