use egui::{Label, Ui};
use toolbox_core::features::network::{
    Connectivity, RadioAccess, RadioKind, RadioTarget, ServiceRow, flight_mode, radio_summary,
    service_rows,
};
use toolbox_core::features::services::{ActiveState, FileState};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{ButtonKind, Tone, banner, card, pill_button, rich, status_dot};
use toolbox_gui_kit::{UiExt as _, icons};

use crate::command::NetworkCommand;
use crate::i18n;
use crate::network::{Address, Read};
use crate::pages::rows::more::{item, more};
use crate::pages::setting_row::{Words, row, stated_row, switch_row};
use crate::pages::{PageContext, heading};
use crate::share::view::copy;

const VALUE_WIDTH: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_narrow;
const START_WIDTH: f32 = 120.0;

#[derive(Clone, Copy)]
enum Value<'a> {
    Text(&'a str),
    State(&'a str, Tone),
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>) {
    connection(ui, context);
    ui.add_space(ui.tokens().sizes.section_gap);
    radios(ui, context);
    ui.add_space(ui.tokens().sizes.section_gap);
    services(ui, context);
}

fn fact(ui: &mut Ui, first: bool, title: &str, value: Value<'_>) {
    let words = Words {
        current: None,
        title,
        description: "",
        note: None,
        queued: None,
    };

    row(ui, first, &words, VALUE_WIDTH, |ui| match value {
        Value::Text(text) => {
            ui.add(Label::new(text).truncate());
        }
        Value::State(word, tone) => {
            status_dot(ui, word, tone);
        }
    });
}

fn connection(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let network = context.network;
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    heading(ui, icons::NETWORK_CONNECT, "network-connection");

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
        .map_or_else(|| missing.clone(), |gateway| gateway.to_string());

    let interface = known
        .and_then(|known| known.interface.clone())
        .unwrap_or(missing);

    card(ui, |ui| {
        let state = text(internet);
        fact(
            ui,
            true,
            &text("network-internet"),
            Value::State(&state, tone),
        );
        fact(ui, false, &text("network-router"), Value::Text(&router));
        fact(
            ui,
            false,
            &text("network-interface"),
            Value::Text(&interface),
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
    let address = context.network.address();
    let shown = match address {
        Address::NotAsked => String::new(),
        Address::Loading => text("network-address-asking"),
        Address::Known(known) => known.to_string(),
        Address::Failed => text("network-address-failed"),
    };

    let title = text("network-address");
    let words = Words {
        current: None,
        title: &title,
        description: &shown,
        note: None,
        queued: None,
    };

    row(ui, false, &words, VALUE_WIDTH, |ui| {
        ui.horizontal(|ui| match address {
            Address::Known(known) => {
                if pill_button(ui, &text("network-address-copy"), ButtonKind::Soft).clicked() {
                    copy(&ctx, context.commands, &known.to_string(), "network-copied");
                }

                more(ui, &title, |ui| {
                    if item(ui, "network-address-hide", true) {
                        context.commands.send(NetworkCommand::ForgetAddress);
                    }
                });
            }
            Address::NotAsked | Address::Failed => {
                let id = if *address == Address::Failed {
                    "network-address-again"
                } else {
                    "network-address-show"
                };

                if pill_button(ui, &text(id), ButtonKind::Soft).clicked() {
                    context.commands.send(NetworkCommand::LookupAddress);
                }
            }
            Address::Loading => {}
        });
    });
}

fn radios(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    heading(ui, icons::PREFERENCES_SYSTEM_BLUETOOTH, "network-radios");
    let status = context.network.status();
    let radios = status
        .and_then(|status| status.radios.known())
        .map_or(&[][..], Vec::as_slice);

    let access = status.map_or(RadioAccess::Allowed, |status| status.access);
    let any = radios.iter().any(|radio| radio.kind.is_some());

    notes(ui, access, any, context.network.radio_failed());
    let allowed = access == RadioAccess::Allowed;
    card(ui, |ui| {
        for (first, kind, target, id) in [
            (true, RadioKind::Wifi, RadioTarget::Wifi, "network-wifi"),
            (
                false,
                RadioKind::Bluetooth,
                RadioTarget::Bluetooth,
                "network-bluetooth",
            ),
        ] {
            let summary = radio_summary(radios, kind);

            // Without the radio device the switch states cannot be read, so
            // a switch never claims on.
            let unknown = access == RadioAccess::Missing && summary.present;
            let detail = if !summary.present {
                i18n::text(&ctx, &format!("{id}-missing")).into_owned()
            } else if unknown {
                i18n::text(&ctx, "network-radio-unknown").into_owned()
            } else if summary.hard {
                i18n::text(&ctx, "network-radio-hard").into_owned()
            } else {
                String::new()
            };

            let enabled = allowed && summary.present && !summary.hard;
            radio(
                ui,
                context,
                (first, id, &detail),
                (summary.present && !summary.blocked && !unknown, enabled),
                |on| NetworkCommand::SetRadio {
                    target,
                    blocked: !on,
                },
            );
        }

        let flight = flight_mode(radios);
        let detail = if any {
            String::new()
        } else {
            i18n::text(&ctx, "network-flight-missing").into_owned()
        };

        radio(
            ui,
            context,
            (false, "network-flight", &detail),
            (flight, allowed && any),
            |on| NetworkCommand::SetRadio {
                target: RadioTarget::All,
                blocked: on,
            },
        );
    });
}

/// One switch row. `command` makes the command for the new state.
fn radio(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (first, id, detail): (bool, &str, &str),
    (on, enabled): (bool, bool),
    command: impl FnOnce(bool) -> NetworkCommand,
) {
    let title = i18n::text(ui.ctx(), id);
    let words = Words {
        current: None,
        title: &title,
        description: detail,
        note: None,
        queued: None,
    };

    if switch_row(ui, first, &words, on, enabled) {
        context.commands.send(command(!on));
    }
}

fn services(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    heading(ui, icons::SYSTEM_RUN, "network-services");
    let listing = &context.units.system;
    let Some(units) = listing.get() else {
        let id = if listing.error().is_some() {
            "network-services-unknown"
        } else {
            "network-services-reading"
        };

        banner(ui, Tone::Neutral, &i18n::text(&ctx, id), &[]);
        return;
    };

    card(ui, |ui| {
        for (index, service) in service_rows(units).iter().enumerate() {
            service_row(ui, context, index == 0, service);
        }
    });
}

fn service_row(ui: &mut Ui, context: &PageContext<'_>, first: bool, service: &ServiceRow) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(
        &ctx,
        &format!("network-service-{}", service.unit.replace('.', "-")),
    );

    let (state_id, tone) = state(service);
    let state = i18n::text(&ctx, state_id);
    let words = Words {
        current: None,
        title: &title,
        description: "",
        note: None,
        queued: None,
    };

    let startable = service.installed
        && matches!(service.active, ActiveState::Inactive | ActiveState::Failed)
        && service.file_state != FileState::Static;

    stated_row(
        ui,
        (first, Some((&state, tone))),
        &words,
        START_WIDTH,
        |ui| {
            if startable {
                let start = i18n::text(&ctx, "network-service-start");
                let clicked = ui
                    .add_enabled_ui(!context.busy, |ui| {
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
    if !service.installed {
        return ("network-service-missing", Tone::Neutral);
    }

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

/// Why the switches cannot be used, and whether the last switch failed.
fn notes(ui: &mut Ui, access: RadioAccess, any: bool, failed: bool) {
    let note = match access {
        RadioAccess::Denied if any => Some("network-radio-denied"),
        RadioAccess::Missing if any => Some("network-radio-missing"),
        RadioAccess::Allowed | RadioAccess::Denied | RadioAccess::Missing => None,
    };

    let banners = note
        .map(|id| (Tone::Warning, id))
        .into_iter()
        .chain(failed.then_some((Tone::Error, "network-radio-failed")));

    for (tone, id) in banners {
        banner(ui, tone, &i18n::text(ui.ctx(), id), &[]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }
}
