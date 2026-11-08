//! The two ways to ask NetworkManager whether the internet is reachable.
//! Not a ping to a public address, which tells nothing behind a firewall
//! that drops pings and sends a packet out on every refresh.

use toolbox_state::network::Connectivity;
use zbus::OwnedVariant;

use crate::bus::Method;

const NETWORK_MANAGER: &str = "org.freedesktop.NetworkManager";
const NETWORK_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager";

/// `CheckConnectivity() -> u`: checks again now and answers the state.
pub const CHECK_CONNECTIVITY: Method = Method {
    destination: NETWORK_MANAGER,
    path: NETWORK_MANAGER_PATH,
    interface: NETWORK_MANAGER,
    name: "CheckConnectivity",
};

/// The `Connectivity` property: the state of the last check, for a
/// session that may not ask for a new one.
pub const CONNECTIVITY_PROPERTY: Method = Method {
    destination: NETWORK_MANAGER,
    path: NETWORK_MANAGER_PATH,
    interface: "org.freedesktop.DBus.Properties",
    name: "Get",
};

/// `Get(interface, property)`.
pub type PropertyBody = (&'static str, &'static str);

pub const fn connectivity_property_body() -> PropertyBody {
    (NETWORK_MANAGER, "Connectivity")
}

/// The state in the variant a property read answers. Anything but an
/// unsigned number counts as unknown.
pub fn connectivity_from_value(value: &OwnedVariant) -> Connectivity {
    u32::try_from(value).map_or(Connectivity::Unknown, Connectivity::from_code)
}
