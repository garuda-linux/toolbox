mod interfaces;
mod radios;

use egui::{Label, Ui};
use toolbox_core::features::network::{Connectivity, NetworkUnit, ServiceRow, service_rows};
use toolbox_core::features::services::{ActiveState, FileState};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, SkeletonRows, Tone, banner, card, icon_button, icon_button_width, pill_button,
    rich, skeleton_list, status_dot,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::UnitScope;

use crate::command::NetworkCommand;
use crate::i18n;
use crate::network::{Address, Network, Read};
use crate::pages::rows::more::{item, more};
use crate::pages::setting_row::{Words, row, stated_row};
use crate::pages::{PageContext, heading};
use crate::share::view::copy;
use crate::units::Units;

const VALUE_WIDTH: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_narrow;
const START_WIDTH: f32 = 120.0;

/// A fact's value at the row's right. A placeholder, such as "Reading…",
/// is dim so it does not read as a value.
#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a str),
    Placeholder(&'a str),
    State(&'a str, Tone),
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>) {
    connection(ui, context);
    ui.add_space(ui.tokens().sizes.section_gap);
    interfaces::show(ui, context);
    radios::show(ui, context);
    ui.add_space(ui.tokens().sizes.section_gap);
    services(ui, context);
}

const fn words(title: &str) -> Words<'_> {
    Words {
        current: None,
        title,
        description: "",
        note: None,
        queued: None,
    }
}

fn fact(ui: &mut Ui, first: bool, title: &str, value: Value<'_>) {
    row(ui, first, &words(title), VALUE_WIDTH, |ui| {
        value_label(ui, value);
    });
}

fn value_label(ui: &mut Ui, value: Value<'_>) {
    let tokens = ui.tokens();
    let (text, ink) = match value {
        Value::Text(text) => (text, tokens.text),
        Value::Placeholder(text) => (text, tokens.secondary),
        Value::State(word, tone) => {
            status_dot(ui, word, tone);
            return;
        }
    };

    ui.add(Label::new(rich(ui, text, Weight::Regular, BODY, ink)).truncate());
}

fn shown_or<'a>(shown: Option<&'a str>, missing: &'a str) -> Value<'a> {
    shown.map_or(Value::Placeholder(missing), Value::Text)
}

fn connection(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let network = context.service::<Network>();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    heading(ui, "network-connection");

    let (internet, tone) = internet_state(network.connectivity());

    let route = network.status().map(|status| &status.route);
    let missing = match route {
        None | Some(Read::Loading) => text("network-reading"),
        Some(Read::Known(_)) => text("network-none"),
        Some(Read::Unavailable | Read::Failed) => text("network-not-known"),
    };

    let known = route.and_then(Read::known);
    let router = known
        .and_then(|known| known.gateway)
        .map(|gateway| gateway.to_string());

    let interface = known.and_then(|known| known.interface.clone());

    card(ui, |ui| {
        let state = text(internet);
        fact(
            ui,
            true,
            &text("network-internet"),
            Value::State(&state, tone),
        );
        fact(
            ui,
            false,
            &text("network-router"),
            shown_or(router.as_deref(), &missing),
        );
        fact(
            ui,
            false,
            &text("network-interface"),
            shown_or(interface.as_deref(), &missing),
        );
        address(ui, context);
    });

    ui.add_space(ui.tokens().sizes.card_gap);
    let secondary = ui.tokens().secondary;
    let note = text("network-address-note");
    ui.add(Label::new(rich(ui, &note, Weight::Regular, SMALL, secondary)).wrap());
}

const fn internet_state(connectivity: Option<Connectivity>) -> (&'static str, Tone) {
    match connectivity {
        None => ("network-internet-checking", Tone::Neutral),
        Some(Connectivity::Full) => ("network-internet-full", Tone::Success),
        Some(Connectivity::Limited) => ("network-internet-limited", Tone::Warning),
        Some(Connectivity::Portal) => ("network-internet-portal", Tone::Warning),
        Some(Connectivity::None) => ("network-internet-none", Tone::Error),
        Some(Connectivity::Unknown) => ("network-internet-unknown", Tone::Neutral),
    }
}

/// Nothing is asked before a click.
fn address(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let address = context.service::<Network>().address();
    let title = text("network-address");

    // Room for an IPv6 address and the two icons behind it.
    let trailing = 2.0 * (icon_button_width() + ui.spacing().item_spacing.x);
    row(ui, false, &words(&title), VALUE_WIDTH + trailing, |ui| {
        ui.horizontal(|ui| match address {
            Address::Known(known) => {
                let shown = known.to_string();
                value_label(ui, Value::Text(&shown));
                if icon_button(ui, icons::EDIT_COPY, &text("network-address-copy")).clicked() {
                    copy(&ctx, context.commands, &shown, "network-copied");
                }

                more(ui, &title, |ui| {
                    if item(ui, "network-address-hide", true) {
                        context.commands.send(NetworkCommand::ForgetAddress);
                    }
                });
            }
            Address::NotAsked | Address::Failed => {
                let id = if *address == Address::Failed {
                    value_label(ui, Value::Placeholder(&text("network-address-failed")));
                    "network-address-again"
                } else {
                    "network-address-show"
                };

                if pill_button(ui, &text(id), ButtonKind::Soft).clicked() {
                    context.commands.send(NetworkCommand::LookupAddress);
                }
            }
            Address::Loading => {
                value_label(ui, Value::Placeholder(&text("network-address-asking")));
            }
        });
    });
}

fn services(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    heading(ui, "network-services");
    let listing = context.service::<Units>().get(UnitScope::System);
    let Some(units) = listing.get() else {
        if listing.error().is_some() {
            banner(
                ui,
                Tone::Neutral,
                &i18n::text(&ctx, "network-services-unknown"),
                &[],
            );
        } else {
            // The rows to come, so the card does not change when they do.
            let name = i18n::text(&ctx, "network-services-reading");
            let rows = SkeletonRows {
                count: 4,
                row_height: ui.tokens().sizes.toolbar_height,
                leading: false,
            };

            card(ui, |ui| skeleton_list(ui, rows, &name));
        }

        return;
    };

    card(ui, |ui| {
        for (index, service) in service_rows(units).iter().enumerate() {
            service_row(ui, context, index == 0, service);
        }
    });
}

/// The ids are literals, so grep and the catalog checks see each one.
const fn service_title_id(unit: NetworkUnit) -> &'static str {
    match unit {
        NetworkUnit::NetworkManager => "network-service-NetworkManager-service",
        NetworkUnit::Connman => "network-service-connman-service",
        NetworkUnit::Bluetooth => "network-service-bluetooth-service",
        NetworkUnit::Geoclue => "network-service-geoclue-service",
    }
}

fn service_row(ui: &mut Ui, context: &PageContext<'_>, first: bool, service: &ServiceRow) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, service_title_id(service.unit));

    let (state_id, tone) = state(service);
    let state = i18n::text(&ctx, state_id);
    let startable = matches!(service.active, ActiveState::Inactive | ActiveState::Failed)
        && service.file_state != FileState::Static;

    stated_row(
        ui,
        (first, Some((&state, tone))),
        &words(&title),
        START_WIDTH,
        |ui| {
            if startable {
                let start = i18n::text(&ctx, "network-service-start");
                let clicked = ui
                    .enabled_ui(!context.busy, |ui| {
                        pill_button(ui, &start, ButtonKind::Soft)
                    })
                    .inner
                    .clicked();

                if clicked {
                    context
                        .commands
                        .send(NetworkCommand::StartService(service.unit));
                }
            }
        },
    );
}

/// The service's state as a text id and the tone of its dot, the tones
/// of the Services page.
const fn state(service: &ServiceRow) -> (&'static str, Tone) {
    match service.active {
        ActiveState::Active => ("network-service-running", Tone::Success),
        ActiveState::Failed => ("network-service-failed", Tone::Error),
        ActiveState::Activating => ("network-service-starting", Tone::Warning),
        ActiveState::Deactivating => ("network-service-stopping", Tone::Warning),
        ActiveState::Reloading => ("network-service-reloading", Tone::Warning),
        ActiveState::Maintenance => ("network-service-cleaning", Tone::Warning),
        ActiveState::Refreshing => ("network-service-refreshing", Tone::Warning),
        ActiveState::Inactive | ActiveState::Other => {
            if matches!(service.file_state, FileState::Static) {
                ("network-service-on-demand", Tone::Neutral)
            } else {
                ("network-service-stopped", Tone::Neutral)
            }
        }
    }
}
