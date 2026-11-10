mod menu;
#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_core::features::boot::kernel_default;
use toolbox_gui_kit::widgets::{Tone, banner, card};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::LoaderDefault;
use toolbox_plan::sdboot::{LoaderSetting, MAX_ENTRIES, SdBootMenu};
use toolbox_system::Menu;

use super::{heading, note};
use crate::boot::Boot;
use crate::command::BootCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::setting_row::{Words, row};

pub fn menu_card(ui: &mut Ui, context: &PageContext<'_>) {
    let menu = context.service::<Boot>().menu();

    heading(ui, icons::FORMAT_LIST_UNORDERED, "boot-menu");

    if let Some(sdboot) = menu.sdboot() {
        counts(ui, sdboot);
    }

    card(ui, |ui| {
        if let Some(sdboot) = menu.sdboot() {
            menu::show(ui, context, sdboot);
        } else {
            // The command line below is known without the menu, so the page
            // stays and this row says why the menu is missing.
            let state = format!("boot-default-{}", menu.as_str());

            fact(ui, "boot-default", &state);
        }
    });
}

fn counts(ui: &mut Ui, menu: &SdBootMenu) {
    if menu.skipped == 0 && menu.cut == 0 {
        return;
    }

    let ctx = ui.ctx().clone();
    let limit = MAX_ENTRIES.to_string();
    let mut sentences = Vec::new();

    if menu.skipped > 0 {
        let count = menu.skipped.to_string();

        sentences.push(i18n::text_with(
            &ctx,
            "boot-sdboot-skipped",
            &[("count", &count)],
        ));
    }

    if menu.cut > 0 {
        let count = menu.cut.to_string();
        let args = [("count", count.as_str()), ("limit", limit.as_str())];

        sentences.push(i18n::text_with(&ctx, "boot-sdboot-cut", &args));
    }

    banner(ui, Tone::Warning, &sentences.join(" "), &[]);
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
            "loading"
        } else {
            "sdboot-none"
        };

        note(ui, context, kernel, state, || {});
        return;
    };

    let wanted = menu::wanted(&sdboot.settings, context).default;
    let default = kernel_default(sdboot, kernel, context.snapshot);
    let state = match &default {
        None => "sdboot-none",
        Some(_) if wanted.as_ref() == Some(&LoaderDefault::saved()) => "remember",
        Some(pattern) if wanted.as_ref() == Some(pattern) => "same",
        Some(_) => "offer",
    };

    note(ui, context, kernel, state, || {
        let setting = LoaderSetting::Default(default);

        context.commands.send(BootCommand::Loader(setting));
    });
}

pub fn no_cmdline_card(ui: &mut Ui) {
    heading(ui, icons::UTILITIES_TERMINAL, "boot-kernel");
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
