//! The interface cards of the Network page, read from NetworkManager over
//! D-Bus: one card per activated Ethernet or Wi-Fi device. The reads are
//! `GetAll` replies behind [`PropertyReads`], so the parser runs on the
//! captures in the tests and on the bus in the app.

mod system;
#[cfg(test)]
mod tests;

use std::future::Future;
use std::net::IpAddr;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
pub use system::{property_changes, system_cards};
use toolbox_state::network::{InterfaceCard, Link, Signal, Subnet};

use crate::bus::CallError;

/// The service and the manager's root object, which the signals of every
/// object below it match.
pub const DESTINATION: &str = "org.freedesktop.NetworkManager";
pub const ROOT_PATH: &str = "/org/freedesktop/NetworkManager";

const DNS_PATH: &str = "/org/freedesktop/NetworkManager/DnsManager";

const MANAGER: &str = "org.freedesktop.NetworkManager";
const DNS: &str = "org.freedesktop.NetworkManager.DnsManager";
const DEVICE: &str = "org.freedesktop.NetworkManager.Device";
const WIRED: &str = "org.freedesktop.NetworkManager.Device.Wired";
const WIRELESS: &str = "org.freedesktop.NetworkManager.Device.Wireless";
const ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";
const IP4: &str = "org.freedesktop.NetworkManager.IP4Config";
const IP6: &str = "org.freedesktop.NetworkManager.IP6Config";
const ACCESS_POINT: &str = "org.freedesktop.NetworkManager.AccessPoint";

/// `NM_DEVICE_STATE_ACTIVATED`, the only state with a card.
const ACTIVATED: u32 = 100;
/// `NM_DEVICE_TYPE_ETHERNET` and `NM_DEVICE_TYPE_WIFI`, the kinds with a card.
const ETHERNET: u32 = 1;
const WIFI: u32 = 2;

/// NetworkManager's placeholder for an object that does not exist.
const NO_OBJECT: &str = "/";

/// Reads the `GetAll` of one NetworkManager object as a plain JSON map of
/// its properties, the busctl reply without its type tags.
pub trait PropertyReads: Sync {
    /// # Errors
    ///
    /// [`CallError::Unavailable`] while NetworkManager does not run or the
    /// object is gone, [`CallError::Failed`] for any other failure.
    fn all(
        &self,
        path: &str,
        interface: &str,
    ) -> impl Future<Output = Result<Value, CallError>> + Send;
}

/// The manager's `Devices`, every device it manages.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Manager {
    devices: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Device {
    interface: String,
    state: u32,
    #[serde(rename = "DeviceType")]
    kind: u32,
    active_connection: String,
    ip4_config: String,
    ip6_config: String,
}

/// The connection a device carries, which the card is titled with.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Active {
    id: String,
    default: bool,
    default6: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Addresses {
    address_data: Vec<AddressRow>,
}

#[derive(Deserialize)]
struct AddressRow {
    address: String,
    prefix: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Dns {
    configuration: Vec<DnsEntry>,
}

/// One server list NetworkManager holds, with the interface it belongs to.
#[derive(Deserialize)]
struct DnsEntry {
    interface: String,
    nameservers: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Wired {
    hw_address: String,
    /// Megabits per second, 0 while the link reports none.
    speed: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Wireless {
    hw_address: String,
    active_access_point: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AccessPoint {
    ssid: Vec<u8>,
    strength: u8,
}

/// The cards of the activated Ethernet and Wi-Fi devices, the one carrying
/// the default connection first. A device whose reads fail is left out
/// with a debug line, as a device can vanish between the list and its read.
///
/// # Errors
///
/// What the manager's read failed with: [`CallError::Unavailable`] when
/// NetworkManager does not run.
pub async fn cards<R>(reads: &R) -> Result<Vec<InterfaceCard>, CallError>
where
    R: PropertyReads,
{
    let manager: Manager = typed(reads.all(ROOT_PATH, MANAGER).await?)?;
    let dns = optional::<Dns, _>(reads, DNS_PATH, DNS)
        .await
        .map_or_else(Vec::new, |dns| dns.configuration);

    let mut found = Vec::new();
    for path in &manager.devices {
        match device_card(reads, path, &dns).await {
            Ok(Some(card)) => found.push(card),
            Ok(None) => {}
            Err(error) => tracing::debug!("network device {path} skipped: {error}"),
        }
    }

    found.sort_by_key(|(default, _)| !default);
    Ok(found.into_iter().map(|(_, card)| card).collect())
}

/// The card of one device with whether it carries the default connection,
/// `None` for a device that is not activated or of another kind.
async fn device_card<R>(
    reads: &R,
    path: &str,
    dns: &[DnsEntry],
) -> Result<Option<(bool, InterfaceCard)>, CallError>
where
    R: PropertyReads,
{
    let device: Device = typed(reads.all(path, DEVICE).await?)?;
    if device.state != ACTIVATED || !matches!(device.kind, ETHERNET | WIFI) {
        return Ok(None);
    }

    let active: Active = typed(reads.all(&device.active_connection, ACTIVE).await?)?;
    let (mac, link) = if device.kind == WIFI {
        wireless(reads, path).await
    } else {
        wired(reads, path).await
    };

    let mut ipv6 = addresses(reads, &device.ip6_config, IP6).await;

    // Global and unique local addresses first, link-local last.
    ipv6.sort_by_key(|subnet| is_link_local(subnet.address));

    let card = InterfaceCard {
        connection: active.id,
        dns: servers(dns, &device.interface),
        ipv4: addresses(reads, &device.ip4_config, IP4).await,
        ipv6,
        interface: device.interface,
        mac,
        link,
    };

    Ok(Some((active.default || active.default6, card)))
}

async fn wired<R>(reads: &R, path: &str) -> (Option<String>, Link)
where
    R: PropertyReads,
{
    let wired: Option<Wired> = optional(reads, path, WIRED).await;
    let mac = wired.as_ref().and_then(|wired| hardware(&wired.hw_address));
    let speed = wired.and_then(|wired| (wired.speed > 0).then_some(wired.speed));

    (mac, Link::Wired { speed })
}

async fn wireless<R>(reads: &R, path: &str) -> (Option<String>, Link)
where
    R: PropertyReads,
{
    let wireless: Option<Wireless> = optional(reads, path, WIRELESS).await;
    let mac = wireless
        .as_ref()
        .and_then(|wireless| hardware(&wireless.hw_address));

    let access = wireless
        .map(|wireless| wireless.active_access_point)
        .filter(|access| access != NO_OBJECT);

    let point: Option<AccessPoint> = match access {
        Some(access) => optional(reads, &access, ACCESS_POINT).await,
        None => None,
    };

    let ssid = point
        .as_ref()
        .map(|point| String::from_utf8_lossy(&point.ssid).into_owned())
        .filter(|ssid| !ssid.is_empty());

    let signal = point.map(|point| Signal {
        percent: point.strength,
    });

    (mac, Link::Wireless { ssid, signal })
}

async fn addresses<R>(reads: &R, path: &str, interface: &str) -> Vec<Subnet>
where
    R: PropertyReads,
{
    if path == NO_OBJECT {
        return Vec::new();
    }

    optional::<Addresses, _>(reads, path, interface)
        .await
        .map_or_else(Vec::new, |addresses| {
            addresses
                .address_data
                .into_iter()
                .filter_map(|row| {
                    let address = row.address.parse::<IpAddr>().ok()?;
                    Some(Subnet {
                        address,
                        prefix: row.prefix,
                    })
                })
                .collect()
        })
}

/// The servers listed for `interface`, IPv4 and IPv6 in the order
/// NetworkManager gives them.
fn servers(entries: &[DnsEntry], interface: &str) -> Vec<IpAddr> {
    let mut servers: Vec<IpAddr> = Vec::new();
    let listed = entries
        .iter()
        .filter(|entry| entry.interface == interface)
        .flat_map(|entry| &entry.nameservers);

    for server in listed {
        if let Ok(address) = server.parse::<IpAddr>()
            && !servers.contains(&address)
        {
            servers.push(address);
        }
    }

    servers
}

const fn is_link_local(address: IpAddr) -> bool {
    matches!(address, IpAddr::V6(address) if address.segments()[0] & 0xffc0 == 0xfe80)
}

/// NetworkManager prints an empty hardware address for a device without one.
fn hardware(address: &str) -> Option<String> {
    (!address.is_empty()).then(|| address.to_owned())
}

/// A property read that fails is logged and leaves its part out, so the
/// card still shows what NetworkManager does give.
async fn optional<T, R>(reads: &R, path: &str, interface: &str) -> Option<T>
where
    T: DeserializeOwned,
    R: PropertyReads,
{
    match reads.all(path, interface).await.and_then(typed) {
        Ok(value) => Some(value),
        Err(error) => {
            tracing::debug!("network {interface} of {path} left out: {error}");
            None
        }
    }
}

fn typed<T>(value: Value) -> Result<T, CallError>
where
    T: DeserializeOwned,
{
    serde_json::from_value(value)
        .map_err(|error| CallError::Failed(format!("unexpected properties: {error}")))
}
