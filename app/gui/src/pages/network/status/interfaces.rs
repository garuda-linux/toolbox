//! One card per interface NetworkManager activated: the connection is the
//! title, the addresses, servers and link are the rows. Nothing shows
//! while NetworkManager does not run.

use egui::Ui;
use toolbox_core::features::network::{InterfaceCard, Link, Signal, SignalQuality};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{PropertyTexts, SectionHeading, card, property_list};

use crate::i18n;
use crate::i18n::Arg;
use crate::network::Network;
use crate::pages::PageContext;

struct Labels {
    interface: String,
    ipv4: String,
    ipv6: String,
    dns: String,
    mac: String,
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(interfaces) = context.service::<Network>().interfaces() else {
        return;
    };

    for interface in interfaces {
        // Each card's rows restart their ids, so a card gets its own scope.
        ui.push_id(&interface.interface, |ui| interface_card(ui, interface));
        ui.add_space(ui.tokens().sizes.section_gap);
    }
}

fn interface_card(ui: &mut Ui, interface: &InterfaceCard) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();

    SectionHeading::new(&interface.connection, "").show(ui);
    ui.add_space(ui.tokens().sizes.heading_gap);

    let copy = |label: &str| {
        i18n::text_with(&ctx, "network-copy", &[("label", Arg::from(label))]).into_owned()
    };

    let copied = text("network-copied");
    let texts = PropertyTexts {
        copy: &copy,
        copied: &copied,
    };

    let labels = Labels {
        interface: text("network-interface"),
        ipv4: text("network-card-ipv4"),
        ipv6: text("network-card-ipv6"),
        dns: text("network-card-dns"),
        mac: text("network-card-mac"),
    };

    let ipv4: Vec<String> = interface.ipv4.iter().map(ToString::to_string).collect();
    let ipv6: Vec<String> = interface.ipv6.iter().map(ToString::to_string).collect();
    let dns: Vec<String> = interface.dns.iter().map(ToString::to_string).collect();
    let link = link_rows(&ctx, &interface.link);

    let padding = ui.tokens().sizes.card_padding;
    card(ui, |ui| {
        egui::Frame::new().inner_margin(padding).show(ui, |ui| {
            property_list(ui, texts, |list| {
                list.row_copied(&labels.interface, &interface.interface);
                for address in &ipv4 {
                    list.row_copied(&labels.ipv4, address);
                }

                for address in &ipv6 {
                    list.row_copied(&labels.ipv6, address);
                }

                for server in &dns {
                    list.row_copied(&labels.dns, server);
                }

                if let Some(mac) = &interface.mac {
                    list.row_copied(&labels.mac, mac.as_str());
                }

                for (label, value, copyable) in &link {
                    if *copyable {
                        list.row_copied(label, value);
                    } else {
                        list.row(label, value);
                    }
                }
            });
        });
    });
}

/// The rows of the link as label, value and whether the value has a copy
/// button: the speed of a wire, or the network name and the signal of Wi-Fi.
fn link_rows(ctx: &egui::Context, link: &Link) -> Vec<(String, String, bool)> {
    let text = |id: &str| i18n::text(ctx, id).into_owned();

    match link {
        Link::Wired { speed: Some(speed) } => vec![(
            text("network-card-speed"),
            i18n::text_with(ctx, "network-speed", &[("mbps", Arg::from(speed))]).into_owned(),
            false,
        )],
        Link::Wired { speed: None } => Vec::new(),
        Link::Wireless { ssid, signal } => {
            let mut rows = Vec::new();
            if let Some(ssid) = ssid {
                rows.push((text("network-card-ssid"), ssid.to_string(), true));
            }

            if let Some(signal) = signal {
                rows.push((
                    text("network-card-signal"),
                    signal_text(ctx, *signal),
                    false,
                ));
            }

            rows
        }
    }
}

/// The signal as its word and the percent, such as "Good (57 %)".
fn signal_text(ctx: &egui::Context, signal: Signal) -> String {
    let word = i18n::text(ctx, quality_id(signal.quality())).into_owned();
    let percent = signal.percent;

    i18n::text_with(
        ctx,
        "network-signal",
        &[("word", Arg::from(&word)), ("percent", Arg::from(percent))],
    )
    .into_owned()
}

const fn quality_id(quality: SignalQuality) -> &'static str {
    match quality {
        SignalQuality::Excellent => "network-signal-excellent",
        SignalQuality::Good => "network-signal-good",
        SignalQuality::Fair => "network-signal-fair",
        SignalQuality::Weak => "network-signal-weak",
    }
}
