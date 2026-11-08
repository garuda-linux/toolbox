mod menu;
#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_core::domains::KernelCmdline;
use toolbox_gui_kit::widgets::card;
use toolbox_gui_kit::{UiExt as _, icons};

use super::heading;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::setting_row::{Words, row};

pub fn show(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let menu = context.boot.menu();

    heading(ui, icons::FORMAT_LIST_UNORDERED, "boot-menu");
    card(ui, |ui| {
        if let Some(sdboot) = menu.sdboot() {
            ui.add_enabled_ui(!context.busy, |ui| menu::show(ui, context, sdboot));
        } else {
            // The command line below is known without the menu, so the page
            // stays and this row says why the menu is missing.
            let state = format!("boot-default-{}", menu.as_str());

            fact(ui, true, "boot-default", &state, None);
        }
    });
    ui.add_space(ui.tokens().sizes.section_gap);
    heading(ui, icons::UTILITIES_TERMINAL, "boot-kernel");
    card(ui, |ui| {
        let line = context.snapshot.part::<KernelCmdline>().map_or_else(
            || i18n::text(&ctx, "boot-sdboot-no-cmdline").into_owned(),
            |cmdline| cmdline.0.to_string(),
        );

        fact(
            ui,
            true,
            "boot-parameters",
            "boot-sdboot-parameters-detail",
            Some(&line),
        );
    });
}

/// The kernel parameters are not changed on systemd-boot yet, so the value
/// is the row's state line instead of a control.
fn fact(ui: &mut Ui, first: bool, title: &str, description: &str, value: Option<&str>) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, title);
    let description = i18n::text(&ctx, description);
    let words = Words {
        title: &title,
        description: &description,
        note: None,
        queued: None,
        current: value,
    };

    row(ui, first, &words, 0.0, |_ui| {});
}
