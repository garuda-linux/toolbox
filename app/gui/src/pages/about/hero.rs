//! The app's identity in the shape of libadwaita's About window: its icon,
//! name and version, who makes it, and the system details to paste into a
//! forum post.

use egui::{Label, Ui, vec2};
use toolbox_gui_kit::fonts::{BODY, TITLE, Weight};
use toolbox_gui_kit::widgets::{ButtonKind, Tone, pill_button, rich, tag};
use toolbox_gui_kit::{Loadable, tokens};
use toolbox_services::home::details::system_details;
use toolbox_system::Host;

use crate::i18n;
use crate::pages::PageContext;
use crate::pages::home::logo;
use crate::share::view::copy;

const ICON_SIZE: f32 = 96.0;

pub fn show(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let name = i18n::text(&ctx, "app-name");
    let credits = i18n::text(&ctx, "about-hero-credits");
    let copy_label = i18n::text(&ctx, "about-copy-details");

    ui.vertical_centered(|ui| {
        ui.spacing_mut().item_spacing.y = tokens.sizes.tight_gap;
        let icon = logo::image(&ctx, tokens.accent, ui.visuals().dark_mode);
        ui.add(icon.fit_to_exact_size(vec2(ICON_SIZE, ICON_SIZE)));
        ui.add_space(tokens.sizes.control_gap);
        ui.add(Label::new(rich(ui, &name, Weight::Bold, TITLE, tokens.text)).wrap());
        tag(ui, context.version, Tone::Neutral);
        ui.add(Label::new(rich(ui, &credits, Weight::Regular, BODY, tokens.secondary)).wrap());
        ui.add_space(tokens.sizes.control_gap);

        if pill_button(ui, &copy_label, ButtonKind::Soft).clicked() {
            let host = context.snapshot.part::<Host>();
            let facts = context.home.facts.and_then(Loadable::get);
            let details = system_details(context.version, host, facts);
            copy(&ctx, context.commands, &details, "about-details-copied");
        }
    });
}
