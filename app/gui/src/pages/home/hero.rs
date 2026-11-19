//! The greeting: whom it greets and what needs attention. The stat tiles
//! below it hold the system's figures.

use egui::{Label, Ui, vec2};
use toolbox_gui_kit::fonts::{BODY, SECTION, Weight};
use toolbox_gui_kit::widgets::{Tone, rich, status_dot, status_dot_then};
use toolbox_gui_kit::{Loadable, UiExt as _, tokens};
use toolbox_services::home::Overview;
use toolbox_system::Host;

use super::logo;
use super::status::attention_rows;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;

const LOGO_SIZE: f32 = 64.0;

pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    host: Option<&Host>,
    overview: Option<&Overview<'_>>,
) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let live = host.is_some_and(|host| host.live_iso);
    let greeting = host.and_then(|host| host.user.as_ref()).map_or_else(
        || i18n::text(&ctx, "home-welcome"),
        |user| i18n::text_with(&ctx, "home-hello", &[("user", Arg::from(user.as_str()))]),
    );

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = ui.tokens().sizes.logo_gap;
        let logo = logo::image(ui.ctx(), tokens.accent, ui.visuals().dark_mode);
        ui.add(logo.fit_to_exact_size(vec2(LOGO_SIZE, LOGO_SIZE)));
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = ui.tokens().sizes.tight_gap;
            ui.add_space(ui.tokens().sizes.tight_gap);
            ui.add(Label::new(rich(ui, &greeting, Weight::Bold, SECTION, tokens.text)).wrap());

            if live {
                subtitle(ui, "home-subtitle");
            } else {
                summary(ui, context, host, overview);
            }
        });
    });
}

fn subtitle(ui: &mut Ui, id: &str) {
    let tokens = tokens::current(ui.ctx());
    let text = i18n::text(ui.ctx(), id);
    ui.label(rich(ui, &text, Weight::Regular, BODY, tokens.secondary));
}

/// The banner under the greeting names checks that could not run, so the
/// greeting does not repeat it and makes no all clear either.
fn summary(
    ui: &mut Ui,
    context: &PageContext<'_>,
    host: Option<&Host>,
    overview: Option<&Overview<'_>>,
) {
    let Some(overview) = overview else {
        let checking = context.home.status.is_some_and(Loadable::is_loading);
        subtitle(
            ui,
            if checking {
                "home-checking"
            } else {
                "home-subtitle"
            },
        );
        return;
    };

    let reboot = host.is_some_and(|host| host.reboot_pending);
    let count = attention_rows(overview, reboot);
    if count == 0 && !overview.failed.is_empty() {
        subtitle(ui, "home-subtitle");
        return;
    }

    let ctx = ui.ctx().clone();
    let (text, tone) = if count == 0 {
        (i18n::text(&ctx, "home-summary-fine"), Tone::Success)
    } else {
        let text = i18n::text_with(
            &ctx,
            "home-summary-attention",
            &[("count", Arg::from(count))],
        );

        (text, Tone::Warning)
    };

    let Some(days) = overview.days else {
        status_dot(ui, &text, tone);
        return;
    };

    let id = if days == 0 {
        "home-summary-updated-today"
    } else {
        "home-summary-updated"
    };

    let updated = i18n::text_with(&ctx, id, &[("days", Arg::from(days))]);
    status_dot_then(ui, &text, &updated, tone);
}
