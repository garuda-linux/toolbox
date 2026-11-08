mod menu;
#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::card;

use super::heading;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::setting_row::{Words, row};

pub fn menu_card(ui: &mut Ui, context: &PageContext<'_>) {
    let menu = context.boot.menu();

    heading(ui, icons::FORMAT_LIST_UNORDERED, "boot-menu");
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
