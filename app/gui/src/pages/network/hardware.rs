//! The Hardware tab: the network devices as lspci and lsusb list them,
//! each line to copy for a forum post.

use egui::Ui;
use toolbox_core::features::network::{Device, DeviceKind, copy_text};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{ButtonKind, Tone, banner, card, empty_state, pill_button, rich};
use toolbox_gui_kit::{UiExt as _, icons};

use crate::i18n;
use crate::network::{Hardware, Network, Read};
use crate::pages::PageContext;
use crate::pages::setting_row::{Words, row};
use crate::share::view::copy;

const COPY_WIDTH: f32 = 150.0;

pub fn show(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let hardware = context.service::<Network>().hardware();
    let devices = hardware.map(devices).unwrap_or_default();

    ui.horizontal(|ui| {
        let secondary = ui.tokens().secondary;
        let intro = text("network-hardware-intro");
        ui.label(rich(ui, &intro, Weight::Regular, SMALL, secondary));
        let all = ui
            .add_enabled_ui(!devices.is_empty(), |ui| {
                pill_button(ui, &text("network-hardware-copy-all"), ButtonKind::Soft)
            })
            .inner;

        if all.clicked() {
            let lines = copy_text(devices.iter().copied());
            copy(&ctx, context.commands, &lines, "network-copied");
        }
    });

    ui.add_space(ui.tokens().sizes.card_gap);
    for note in hardware.map(notes).unwrap_or_default() {
        banner(ui, Tone::Warning, &text(note), &[]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }

    if devices.is_empty() {
        let title = match hardware {
            None => "network-hardware-reading",
            Some(lists) if lists.pci.known().is_none() && lists.usb.known().is_none() => {
                "network-hardware-unreadable"
            }
            Some(_) => "network-hardware-none",
        };

        empty_state(ui, icons::NETWORK_WIRED, &text(title), "");
        return;
    }

    card(ui, |ui| {
        for (index, device) in devices.iter().enumerate() {
            line(ui, context, index == 0, device);
        }
    });
}

/// The PCI devices, then the USB ones.
fn devices(hardware: &Hardware) -> Vec<&Device> {
    [&hardware.pci, &hardware.usb]
        .into_iter()
        .filter_map(Read::known)
        .flatten()
        .collect()
}

/// Why a list is missing. A list still being read says nothing.
fn notes(hardware: &Hardware) -> Vec<&'static str> {
    let pci = match hardware.pci {
        Read::Unavailable => Some("network-hardware-pci-missing"),
        Read::Failed => Some("network-hardware-pci-failed"),
        Read::Loading | Read::Known(_) => None,
    };

    let usb = match hardware.usb {
        Read::Unavailable => Some("network-hardware-usb-missing"),
        Read::Failed => Some("network-hardware-usb-failed"),
        Read::Loading | Read::Known(_) => None,
    };

    pci.into_iter().chain(usb).collect()
}

fn line(ui: &mut Ui, context: &PageContext<'_>, first: bool, device: &Device) {
    let ctx = ui.ctx().clone();
    let kind = i18n::text(&ctx, kind_id(device.kind));
    let words = Words {
        current: None,
        title: &device.description,
        description: &kind,
        note: None,
        queued: None,
    };

    row(ui, first, &words, COPY_WIDTH, |ui| {
        let name = i18n::text(&ctx, "network-hardware-copy-line");
        if pill_button(ui, &name, ButtonKind::Soft).clicked() {
            copy(&ctx, context.commands, &device.raw, "network-copied");
        }
    });
}

const fn kind_id(kind: DeviceKind) -> &'static str {
    match kind {
        DeviceKind::Ethernet => "network-kind-ethernet",
        DeviceKind::Wireless => "network-kind-wireless",
        DeviceKind::Bluetooth => "network-kind-bluetooth",
    }
}
