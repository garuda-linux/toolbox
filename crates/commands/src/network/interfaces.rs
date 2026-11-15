//! The interface cards of `network show`: plain values for JSON, and the
//! facts of each card for the text output.

use serde::Serialize;
use toolbox_cli::output::Paint;
use toolbox_core::features::network::{InterfaceCard, Link, Signal, SignalQuality};
use toolbox_services::i18n::{FALLBACK, text_in, text_with_in};

/// An interface card as the command shows it. The copy buttons of the
/// window have no counterpart here, the values are plain text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct InterfaceView {
    pub connection: String,
    pub interface: String,
    pub mac: Option<String>,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub dns: Vec<String>,
    /// Megabits per second of a wire, `None` while the link reports none.
    pub speed: Option<u32>,
    pub ssid: Option<String>,
    /// Percent of the active access point.
    pub signal: Option<u8>,
}

/// The cards as the command's views, in the order the cards come.
pub fn interface_views(cards: &[InterfaceCard]) -> Vec<InterfaceView> {
    cards
        .iter()
        .map(|card| {
            let (speed, ssid, signal) = match &card.link {
                Link::Wired { speed } => (*speed, None, None),
                Link::Wireless { ssid, signal } => {
                    (None, ssid.clone(), signal.map(|signal| signal.percent))
                }
            };

            InterfaceView {
                connection: card.connection.clone(),
                interface: card.interface.clone(),
                mac: card.mac.clone(),
                ipv4: card.ipv4.iter().map(ToString::to_string).collect(),
                ipv6: card.ipv6.iter().map(ToString::to_string).collect(),
                dns: card.dns.iter().map(ToString::to_string).collect(),
                speed,
                ssid,
                signal,
            }
        })
        .collect()
}

/// A card's facts: the connection as the title, then the rows the card
/// has. Several addresses or servers share a row, separated by commas.
pub fn interface_facts(paint: Paint, card: &InterfaceView) -> String {
    let mut rows: Vec<(&str, String)> = vec![("interface", card.interface.clone())];
    if !card.ipv4.is_empty() {
        rows.push(("ipv4", card.ipv4.join(", ")));
    }

    if !card.ipv6.is_empty() {
        rows.push(("ipv6", card.ipv6.join(", ")));
    }

    if !card.dns.is_empty() {
        rows.push(("dns", card.dns.join(", ")));
    }

    if let Some(mac) = &card.mac {
        rows.push(("mac", mac.clone()));
    }

    if let Some(speed) = card.speed {
        rows.push(("speed", format!("{speed} Mb/s")));
    }

    if let Some(ssid) = &card.ssid {
        rows.push(("ssid", ssid.clone()));
    }

    if let Some(percent) = card.signal {
        rows.push(("signal", signal_text(percent)));
    }

    let rows: Vec<(&str, &str)> = rows
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();

    paint.facts(&card.connection, &rows)
}

/// The signal as the window words it, such as "Good (57 %)", from the same
/// Fluent texts. The thresholds are nmcli's.
pub fn signal_text(percent: u8) -> String {
    let quality = Signal { percent }.quality();
    let id = match quality {
        SignalQuality::Excellent => "network-signal-excellent",
        SignalQuality::Good => "network-signal-good",
        SignalQuality::Fair => "network-signal-fair",
        SignalQuality::Weak => "network-signal-weak",
    };

    let word = text_in(FALLBACK, id).into_owned();
    let percent = percent.to_string();

    text_with_in(
        &FALLBACK,
        "network-signal",
        &[("word", word.as_str()), ("percent", percent.as_str())],
    )
    .into_owned()
}
