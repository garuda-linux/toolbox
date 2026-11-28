mod menu;
#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_core::features::boot::{kernel_default, wanted_loader};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{Tone, card};
use toolbox_plan::LoaderDefault;
use toolbox_plan::sdboot::{LoaderSetting, MAX_ENTRIES, SdBootMenu};
use toolbox_services::boot_menu::Menu;

use super::{Proposal, heading, note};
use crate::boot::Boot;
use crate::command::BootCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::rows::read_again_banner;
use crate::pages::setting_row::{Words, row};

pub(super) fn menu_card(ui: &mut Ui, context: &PageContext<'_>) {
    let menu = context.service::<Boot>().menu();

    heading(ui, "boot-menu");

    let state = super::menu::state_id(menu);
    let Some(sdboot) = menu.sdboot() else {
        // The command line below is known without the menu, so the page
        // stays and only this part says why the menu is missing.
        if matches!(menu, Menu::Read(_)) {
            super::menu::unread_banner(ui, context, menu);
        } else {
            card(ui, |ui| fact(ui, "boot-default", state));
        }

        return;
    };

    counts(ui, context, sdboot);
    card(ui, |ui| menu::show(ui, context, sdboot));
}

fn counts(ui: &mut Ui, context: &PageContext<'_>, menu: &SdBootMenu) {
    if menu.skipped == 0 && menu.cut == 0 {
        return;
    }

    let ctx = ui.ctx().clone();
    let mut sentences = Vec::new();

    if menu.skipped > 0 {
        sentences.push(i18n::text_with(
            &ctx,
            "boot-sdboot-skipped",
            &[("count", Arg::from(menu.skipped))],
        ));
    }

    if menu.cut > 0 {
        let args = [
            ("count", Arg::from(menu.cut)),
            ("limit", Arg::from(MAX_ENTRIES)),
        ];

        sentences.push(i18n::text_with(&ctx, "boot-sdboot-cut", &args));
    }

    // Only unread entries can read again, a cut menu shows its first 256.
    let text = sentences.join(" ");
    if read_again_banner(ui, Tone::Warning, &text, menu.skipped > 0) {
        context.commands.send(BootCommand::Refresh);
    }

    ui.add_space(ui.tokens().sizes.card_gap);
}

/// The kernel's image is saved by its flavour, as the menu card saves it.
pub fn proposal(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(kernel) = context.service::<Boot>().proposal() else {
        return;
    };

    let menu = context.service::<Boot>().menu();
    let Some(sdboot) = menu.sdboot() else {
        let state = if matches!(menu, Menu::NotAsked | Menu::Loading) {
            Proposal::Loading
        } else {
            Proposal::SdBootEmpty
        };

        note(ui, context, kernel, state, || {});
        return;
    };

    let wanted = wanted_loader(&sdboot.settings, context.changes).default;
    let default = kernel_default(sdboot, kernel, context.snapshot);
    let state = match &default {
        None => Proposal::SdBootEmpty,
        Some(_) if wanted.as_ref() == Some(&LoaderDefault::saved()) => Proposal::Remember,
        Some(pattern) if wanted.as_ref() == Some(pattern) => Proposal::Same,
        Some(_) => Proposal::Offer,
    };

    note(ui, context, kernel, state, || {
        let setting = LoaderSetting::Default(default);

        context.commands.send(BootCommand::Loader(setting));
    });
}

pub(super) fn no_cmdline_card(ui: &mut Ui) {
    heading(ui, "boot-kernel");
    card(ui, |ui| {
        fact(ui, "boot-parameters", "boot-sdboot-no-cmdline");
    });
}

fn fact(ui: &mut Ui, title: &str, description: &str) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, title);
    let description = i18n::text(&ctx, description);
    let words = Words {
        title: &title,
        description: &description,
        note: None,
        queued: None,
        current: None,
    };

    row(ui, true, &words, 0.0, |_ui| {});
}
