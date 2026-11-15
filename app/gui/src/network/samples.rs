//! Sample interface cards for the self-test and the page tests.
//!
//! A Wi-Fi card with its signal and an Ethernet card with its link speed.
//! They are values, so the self-test shows the same cards on every machine.

use std::net::{IpAddr, Ipv6Addr};

use toolbox_core::features::network::{InterfaceCard, Link, Signal, Subnet};

pub fn cards() -> Vec<InterfaceCard> {
    vec![wifi(), ethernet()]
}

pub fn wifi() -> InterfaceCard {
    InterfaceCard {
        connection: "network-3".to_owned(),
        interface: "wlan0".to_owned(),
        mac: Some("52:54:00:00:00:07".to_owned()),
        ipv4: vec![Subnet {
            address: IpAddr::from([192, 168, 178, 30]),
            prefix: 24,
        }],
        ipv6: vec![
            Subnet {
                address: IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2)),
                prefix: 64,
            },
            Subnet {
                address: IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0xd)),
                prefix: 64,
            },
        ],
        dns: vec![IpAddr::from([192, 168, 178, 1])],
        link: Link::Wireless {
            ssid: Some("network-3".to_owned()),
            signal: Some(Signal { percent: 57 }),
        },
    }
}

pub fn ethernet() -> InterfaceCard {
    InterfaceCard {
        connection: "Wired connection 1".to_owned(),
        interface: "enp3s0".to_owned(),
        mac: Some("52:54:00:00:00:0e".to_owned()),
        ipv4: vec![Subnet {
            address: IpAddr::from([192, 168, 178, 40]),
            prefix: 24,
        }],
        ipv6: Vec::new(),
        dns: vec![IpAddr::from([192, 168, 178, 1])],
        link: Link::Wired { speed: Some(1000) },
    }
}
