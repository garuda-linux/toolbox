//! The greeting: whom it greets and what needs attention. The stat tiles
//! below it hold the system's figures.

use egui::{Label, Ui, vec2};
use toolbox_gui_kit::fonts::{self, BODY, SECTION, SMALL, Weight};
use toolbox_gui_kit::widgets::{Tone, rich, status_dot};
use toolbox_gui_kit::{Loadable, UiExt as _, tokens};
use toolbox_system::Host;

use super::logo;
use super::status::attention_rows;
use crate::i18n;
use crate::pages::PageContext;

const LOGO_SIZE: f32 = 64.0;

pub fn show(ui: &mut Ui, context: &PageContext<'_>, host: Option<&Host>) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let live = host.is_some_and(|host| host.live_iso);
    let greeting = host.and_then(|host| host.user.as_ref()).map_or_else(
        || i18n::text(&ctx, "home-welcome"),
        |user| i18n::text_with(&ctx, "home-hello", &[("user", user.as_str())]),
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
                summary(ui, context, host);
            }
        });
    });
}

fn subtitle(ui: &mut Ui, id: &str) {
    let tokens = tokens::current(ui.ctx());
    let text = i18n::text(ui.ctx(), id);
    ui.label(rich(ui, &text, Weight::Regular, BODY, tokens.secondary));
}

fn summary(ui: &mut Ui, context: &PageContext<'_>, host: Option<&Host>) {
    let reading = context.home.status;
    let Some(status) = reading.and_then(Loadable::get) else {
        let checking = reading.is_some_and(Loadable::is_loading);
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

    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let overview = super::status::overview(status, context);

    let reboot = host.is_some_and(|host| host.reboot_pending);
    let count = attention_rows(&overview, reboot);
    let (text, tone) = if count == 0 && !overview.failed.is_empty() {
        // Nothing found is no all clear while some checks could not run.
        (i18n::text(&ctx, "home-summary-unknown"), Tone::Warning)
    } else if count == 0 {
        (i18n::text(&ctx, "home-summary-fine"), Tone::Success)
    } else {
        let count = count.to_string();
        let text = i18n::text_with(&ctx, "home-summary-attention", &[("count", &count)]);
        (text, Tone::Warning)
    };

    ui.horizontal_wrapped(|ui| {
        // The second part starts with "·" and a space, so a space before
        // it keeps the dot in the middle.
        let font = fonts::font(ui.style(), Weight::Regular, SMALL);
        ui.spacing_mut().item_spacing.x = ui.fonts_mut(|fonts| fonts.glyph_width(&font, ' '));
        status_dot(ui, &text, tone);
        if let Some(days) = overview.days {
            let id = if days == 0 {
                "home-summary-updated-today"
            } else {
                "home-summary-updated"
            };

            let days = days.to_string();
            let updated = i18n::text_with(&ctx, id, &[("days", &days)]);
            ui.label(rich(ui, &updated, Weight::Regular, SMALL, tokens.secondary));
        }
    });
}
