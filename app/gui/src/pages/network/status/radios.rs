//! The radios card: a switch for each radio the machine has, and a plain
//! row for one it lacks, as a switch that can never move only misleads.

use egui::Ui;
use toolbox_core::features::network::{
    RadioAccess, RadioKind, RadioSummary, RadioTarget, flight_mode, radio_summary,
};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{Tone, banner, card};

use crate::command::NetworkCommand;
use crate::i18n;
use crate::network::Network;
use crate::pages::setting_row::{Words, row, switch_row};
use crate::pages::{PageContext, heading};

/// One radio row: its title id, the note under it, and its switch as
/// whether it is on and can move. `None` for a radio the machine lacks.
struct Radio<'a> {
    first: bool,
    id: &'a str,
    detail: &'a str,
    switch: Option<(bool, bool)>,
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    heading(ui, "network-radios");
    let status = context.service::<Network>().status();
    let radios = status
        .and_then(|status| status.radios.known())
        .map_or(&[][..], Vec::as_slice);

    let access = status.map_or(RadioAccess::Allowed, |status| status.access);
    let any = radios.iter().any(|radio| radio.kind.is_some());

    notes(ui, access, any, context.service::<Network>().radio_failed());
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
            let detail = radio_detail(&ctx, id, summary, unknown);

            let switch = summary
                .present
                .then_some((!summary.blocked && !unknown, allowed && !summary.hard));

            let radio = Radio {
                first,
                id,
                detail: &detail,
                switch,
            };

            show_radio(ui, context, &radio, |on| NetworkCommand::SetRadio {
                target,
                blocked: !on,
            });
        }

        let detail = if any {
            String::new()
        } else {
            i18n::text(&ctx, "network-flight-missing").into_owned()
        };

        let flight = Radio {
            first: false,
            id: "network-flight",
            detail: &detail,
            switch: any.then_some((flight_mode(radios), allowed)),
        };

        show_radio(ui, context, &flight, |on| NetworkCommand::SetRadio {
            target: RadioTarget::All,
            blocked: on,
        });
    });
}

fn radio_detail(ctx: &egui::Context, id: &str, summary: RadioSummary, unknown: bool) -> String {
    if !summary.present {
        i18n::text(ctx, &format!("{id}-missing")).into_owned()
    } else if unknown {
        i18n::text(ctx, "network-radio-unknown").into_owned()
    } else if summary.hard {
        i18n::text(ctx, "network-radio-hard").into_owned()
    } else {
        String::new()
    }
}

fn show_radio(
    ui: &mut Ui,
    context: &PageContext<'_>,
    radio: &Radio<'_>,
    command: impl FnOnce(bool) -> NetworkCommand,
) {
    let title = i18n::text(ui.ctx(), radio.id);
    let words = Words {
        current: None,
        title: &title,
        description: radio.detail,
        note: None,
        queued: None,
    };

    let Some((on, enabled)) = radio.switch else {
        row(ui, radio.first, &words, 0.0, |_| {});
        return;
    };

    if switch_row(ui, radio.first, &words, on, enabled) {
        context.commands.send(command(!on));
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
