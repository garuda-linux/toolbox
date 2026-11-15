//! The Hardware tab: the network devices as lspci and lsusb list them,
//! each line to copy for a forum post.

use egui::{Frame, Label, Margin, Ui};
use toolbox_core::Toggle;
use toolbox_core::features::network::{
    Device, DeviceKind, PCIUTILS_PACKAGE, PackageState, copy_text, package_state,
};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, Tone, banner, card, empty_state, empty_state_action, pill_button, pill_width, rich,
    split_row,
};
use toolbox_gui_kit::{UiExt as _, icons};

use crate::command::PackagesCommand;
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
    let reasons: Vec<String> = hardware
        .map(notes)
        .unwrap_or_default()
        .into_iter()
        .map(text)
        .collect();

    if devices.is_empty() {
        let title = match hardware {
            None => "network-hardware-reading",
            Some(lists) if lists.pci.known().is_none() && lists.usb.known().is_none() => {
                "network-hardware-unreadable"
            }
            Some(_) => "network-hardware-none",
        };

        // One state says all: each reason is a line of its body.
        let body = reasons.join("\n");
        match pciutils_action(hardware, context) {
            Some((action, wish)) => {
                if empty_state_action(ui, icons::NETWORK_WIRED, &text(title), &body, &text(action))
                {
                    queue_pciutils(context, wish);
                }
            }
            None => {
                empty_state(ui, icons::NETWORK_WIRED, &text(title), &body);
            }
        }

        return;
    }

    intro(ui, context, &devices);
    ui.add_space(ui.tokens().sizes.card_gap);
    let action = pciutils_action(hardware, context);
    for (index, note) in reasons.iter().enumerate() {
        // The PCI note comes first, the only one a package helps.
        let offered = action.filter(|_| index == 0);
        let words: Vec<String> = offered.iter().map(|(id, _)| text(id)).collect();
        let actions: Vec<&str> = words.iter().map(String::as_str).collect();
        if banner(ui, Tone::Warning, note, &actions) == Some(0)
            && let Some((_, wish)) = offered
        {
            queue_pciutils(context, wish);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }

    card(ui, |ui| {
        for (index, device) in devices.iter().enumerate() {
            line(ui, context, index == 0, device);
        }
    });
}

/// The intro wraps at the left, Copy all ends at the right edge as each
/// line's button does.
fn intro(ui: &mut Ui, context: &PageContext<'_>, devices: &[&Device]) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let all = text("network-hardware-copy-all");

    // The lines' buttons sit in from the card's edge by its padding.
    let inset = Margin {
        right: ui.tokens().sizes.card_padding.right,
        ..Margin::ZERO
    };

    Frame::new().inner_margin(inset).show(ui, |ui| {
        split_row(
            ui,
            pill_width(ui, &all),
            |ui| {
                let secondary = ui.tokens().secondary;
                let intro = text("network-hardware-intro");
                ui.add(Label::new(rich(ui, &intro, Weight::Regular, SMALL, secondary)).wrap());
            },
            |ui| {
                if pill_button(ui, &all, ButtonKind::Soft).clicked() {
                    let lines = copy_text(devices.iter().copied());
                    copy(&ctx, context.commands, &lines, "network-copied");
                }
            },
        );
    });
}

fn pci_missing(hardware: Option<&Hardware>) -> bool {
    hardware.is_some_and(|lists| lists.pci == Read::Unavailable)
}

/// The button that queues pciutils, or takes it back, while lspci is
/// missing. `None` once it is installed or nothing is known.
fn pciutils_action(
    hardware: Option<&Hardware>,
    context: &PageContext<'_>,
) -> Option<(&'static str, Toggle)> {
    if !pci_missing(hardware) {
        return None;
    }

    match package_state(&PCIUTILS_PACKAGE, context.snapshot, context.changes) {
        PackageState::Missing => Some(("network-hardware-queue-pciutils", Toggle::On)),
        PackageState::Queued => Some(("network-hardware-unqueue-pciutils", Toggle::Off)),
        PackageState::Installed | PackageState::Unknown => None,
    }
}

fn queue_pciutils(context: &PageContext<'_>, wish: Toggle) {
    context.commands.send(PackagesCommand::Toggle {
        package: PCIUTILS_PACKAGE.clone(),
        wish,
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
