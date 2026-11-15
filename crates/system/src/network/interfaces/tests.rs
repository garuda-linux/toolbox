use std::future::Future;
use std::net::IpAddr;

use serde_json::Value;
use toolbox_state::network::{InterfaceCard, Link, Signal, SignalQuality, Subnet};

use super::{
    ACCESS_POINT, ACTIVE, DEVICE, DNS, DNS_PATH, IP4, IP6, MANAGER, PropertyReads, ROOT_PATH,
    WIRED, WIRELESS, cards,
};
use crate::bus::CallError;

const DEVICES: &str = "/org/freedesktop/NetworkManager/Devices/";
const ACTIVE_CONNECTIONS: &str = "/org/freedesktop/NetworkManager/ActiveConnection/";
const IP4_CONFIGS: &str = "/org/freedesktop/NetworkManager/IP4Config/";
const IP6_CONFIGS: &str = "/org/freedesktop/NetworkManager/IP6Config/";
const ACCESS_POINTS: &str = "/org/freedesktop/NetworkManager/AccessPoint/";

/// The bare-metal laptop: wlan0 is activated on Wi-Fi, a tun and a bridge
/// are activated too, two Ethernet devices are disconnected. The device
/// paths come from the active connections' device lists. Devices without
/// a capture of their path are left out of the table, as NetworkManager
/// answers for a device that vanished.
const BARE_METAL_MANAGER: &str = include_str!("../fixtures/bare-metal/networkmanager/manager.json");
const BARE_METAL_DNS: &str = include_str!("../fixtures/bare-metal/networkmanager/dns-manager.json");
const WLAN0: &str = include_str!("../fixtures/bare-metal/networkmanager/device-wlan0.json");
const WLAN0_WIRELESS: &str =
    include_str!("../fixtures/bare-metal/networkmanager/device-wlan0-wireless.json");
const WLAN0_ACTIVE: &str = include_str!("../fixtures/bare-metal/networkmanager/active-2.json");
const WLAN0_IP4: &str = include_str!("../fixtures/bare-metal/networkmanager/ip4-wlan0.json");
const WLAN0_IP6: &str = include_str!("../fixtures/bare-metal/networkmanager/ip6-wlan0.json");
const ACCESS_POINT_5: &str = include_str!("../fixtures/bare-metal/networkmanager/ap-wlan0-5.json");
const TAILSCALE: &str =
    include_str!("../fixtures/bare-metal/networkmanager/device-tailscale0.json");
const BRIDGE: &str =
    include_str!("../fixtures/bare-metal/networkmanager/device-br-3f9bfdcd4798.json");
const ENP3S0: &str = include_str!("../fixtures/bare-metal/networkmanager/device-enp3s0.json");
const ENP3S0_WIRED: &str =
    include_str!("../fixtures/bare-metal/networkmanager/device-enp3s0-wired.json");
const ENP3S0_IP4: &str = include_str!("../fixtures/bare-metal/networkmanager/ip4-enp3s0.json");
const ENP3S0_IP6: &str = include_str!("../fixtures/bare-metal/networkmanager/ip6-enp3s0.json");

/// The Lima VM: NetworkManager manages nothing, both devices are unmanaged.
const INSTALLED_MANAGER: &str = include_str!("../fixtures/installed/networkmanager/manager.json");
const INSTALLED_DNS: &str = include_str!("../fixtures/installed/networkmanager/dns-manager.json");
const INSTALLED_ETH0: &str = include_str!("../fixtures/installed/networkmanager/device-eth0.json");
const INSTALLED_LO: &str = include_str!("../fixtures/installed/networkmanager/device-lo.json");

/// What NetworkManager answered for each object and interface.
type Table = Vec<(String, &'static str, String)>;

/// The reads of a [`Table`]. An object not in it is answered as gone.
struct Replies(Table);

impl PropertyReads for Replies {
    fn all(
        &self,
        path: &str,
        interface: &str,
    ) -> impl Future<Output = Result<Value, CallError>> + Send {
        let reply = self
            .0
            .iter()
            .find(|(object, name, _)| object == path && *name == interface)
            .map(|(_, _, json)| reply(json))
            .ok_or(CallError::Unavailable);

        std::future::ready(reply)
    }
}

/// busctl's JSON without its types: the one dict of the reply, with each
/// typed value replaced by its data.
fn reply(json: &str) -> Value {
    let reply: Value = serde_json::from_str(json).unwrap();
    plain(&reply["data"][0])
}

fn plain(value: &Value) -> Value {
    match value {
        Value::Object(map) if map.len() == 2 && map.contains_key("type") => plain(&map["data"]),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, item)| (key.clone(), plain(item)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(plain).collect()),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => value.clone(),
    }
}

/// `json` with `from` replaced by `to`. A replacement that finds nothing
/// fails the test, so an edge case never silently tests the plain capture.
fn changed(json: &str, from: &str, to: &str) -> String {
    assert!(json.contains(from), "the capture has no {from}");
    json.replacen(from, to, 1)
}

fn object(path: &str, name: &'static str, json: &str) -> (String, &'static str, String) {
    (path.to_owned(), name, json.to_owned())
}

/// The bare-metal laptop's replies: the manager, the DNS list and the
/// three devices with a capture of their path.
fn bare_metal() -> Table {
    vec![
        object(ROOT_PATH, MANAGER, BARE_METAL_MANAGER),
        object(DNS_PATH, DNS, BARE_METAL_DNS),
        object(&format!("{DEVICES}4"), DEVICE, WLAN0),
        object(&format!("{DEVICES}4"), WIRELESS, WLAN0_WIRELESS),
        object(&format!("{ACTIVE_CONNECTIONS}3"), ACTIVE, WLAN0_ACTIVE),
        object(&format!("{IP4_CONFIGS}4"), IP4, WLAN0_IP4),
        object(&format!("{IP6_CONFIGS}4"), IP6, WLAN0_IP6),
        object(&format!("{ACCESS_POINTS}5"), ACCESS_POINT, ACCESS_POINT_5),
        object(&format!("{DEVICES}8"), DEVICE, TAILSCALE),
        object(&format!("{DEVICES}9"), DEVICE, BRIDGE),
    ]
}

fn ip(text: &str) -> IpAddr {
    text.parse().unwrap()
}

fn subnet(text: &str, prefix: u8) -> Subnet {
    Subnet {
        address: ip(text),
        prefix,
    }
}

fn wlan0_card() -> InterfaceCard {
    InterfaceCard {
        connection: "network-3".to_owned(),
        interface: "wlan0".to_owned(),
        mac: Some("52:54:00:00:00:07".to_owned()),
        ipv4: vec![subnet("192.168.178.30", 24)],
        ipv6: vec![
            subnet("2001:db8::2", 64),
            subnet("2001:db8::1", 64),
            subnet("fd00::9", 64),
            subnet("fd00::8", 64),
            subnet("fe80::d", 64),
        ],
        dns: vec![ip("192.168.178.1"), ip("fd00::a"), ip("2001:db8::4")],
        link: Link::Wireless {
            ssid: Some("network-3".to_owned()),
            signal: Some(Signal { percent: 57 }),
        },
    }
}

#[tokio::test]
async fn bare_metal_gives_the_wifi_card() {
    let cards = cards(&Replies(bare_metal())).await.unwrap();

    assert_eq!(cards, vec![wlan0_card()]);
    let Link::Wireless {
        signal: Some(signal),
        ssid: _,
    } = &cards[0].link
    else {
        panic!("the card is Wi-Fi with a signal");
    };

    assert_eq!(signal.quality(), SignalQuality::Good);
}

#[tokio::test]
async fn installed_gives_no_cards() {
    let table = vec![
        object(ROOT_PATH, MANAGER, INSTALLED_MANAGER),
        object(DNS_PATH, DNS, INSTALLED_DNS),
        object(&format!("{DEVICES}1"), DEVICE, INSTALLED_LO),
        object(&format!("{DEVICES}2"), DEVICE, INSTALLED_ETH0),
    ];

    assert_eq!(cards(&Replies(table)).await.unwrap(), Vec::new());
}

#[tokio::test]
async fn without_networkmanager_the_read_fails() {
    let error = cards(&Replies(Vec::new())).await.unwrap_err();

    assert_eq!(error, CallError::Unavailable);
}

/// The enp3s0 device has no capture of its path, so it is given
/// `Devices/3`, a path of the manager's list. The changes make it
/// activated on connection 2 with a 1000 Mb/s link. Its connection is the
/// wlan0 capture renamed to `wired-1` and no longer the default, so
/// wlan0 stays first.
#[tokio::test]
async fn an_activated_ethernet_device_gets_its_speed() {
    let mut table = bare_metal();
    let wired_path = format!("{DEVICES}3");
    let connection = format!("{ACTIVE_CONNECTIONS}2");

    let activated = changed(
        ENP3S0,
        r#""State":{"type":"u","data":20}"#,
        r#""State":{"type":"u","data":100}"#,
    );

    let on_connection = changed(
        &activated,
        r#""ActiveConnection":{"type":"o","data":"/"}"#,
        &format!(r#""ActiveConnection":{{"type":"o","data":"{connection}"}}"#),
    );

    table.push(object(&wired_path, DEVICE, &on_connection));

    let fast = changed(
        ENP3S0_WIRED,
        r#""Speed":{"type":"u","data":0}"#,
        r#""Speed":{"type":"u","data":1000}"#,
    );

    table.push(object(&wired_path, WIRED, &fast));

    let renamed = changed(
        WLAN0_ACTIVE,
        r#""Id":{"type":"s","data":"network-3"}"#,
        r#""Id":{"type":"s","data":"wired-1"}"#,
    );

    let not_default = changed(
        &changed(
            &renamed,
            r#""Default":{"type":"b","data":true}"#,
            r#""Default":{"type":"b","data":false}"#,
        ),
        r#""Default6":{"type":"b","data":true}"#,
        r#""Default6":{"type":"b","data":false}"#,
    );

    table.push(object(&connection, ACTIVE, &not_default));
    table.push(object(&format!("{IP4_CONFIGS}3"), IP4, ENP3S0_IP4));
    table.push(object(&format!("{IP6_CONFIGS}3"), IP6, ENP3S0_IP6));

    let cards = cards(&Replies(table)).await.unwrap();

    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0], wlan0_card());
    assert_eq!(cards[1].connection, "wired-1");
    assert_eq!(cards[1].interface, "enp3s0");
    assert_eq!(cards[1].mac.as_deref(), Some("52:54:00:00:00:0E"));
    assert_eq!(cards[1].link, Link::Wired { speed: Some(1000) });
}

#[test]
fn signal_words_follow_nmcli_thresholds() {
    let quality = |percent| Signal { percent }.quality();

    assert_eq!(quality(100), SignalQuality::Excellent);
    assert_eq!(quality(81), SignalQuality::Excellent);
    assert_eq!(quality(80), SignalQuality::Good);
    assert_eq!(quality(56), SignalQuality::Good);
    assert_eq!(quality(55), SignalQuality::Fair);
    assert_eq!(quality(31), SignalQuality::Fair);
    assert_eq!(quality(30), SignalQuality::Weak);
    assert_eq!(quality(0), SignalQuality::Weak);
}
