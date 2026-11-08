#[cfg(test)]
mod tests;

use egui::Ui;
use toolbox_core::domains::KernelCmdline;
use toolbox_gui_kit::widgets::card;
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::sdboot::{LoaderTimeout, SdBootMenu};

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
            menu_rows(ui, sdboot);
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

fn menu_rows(ui: &mut Ui, menu: &SdBootMenu) {
    let ctx = ui.ctx().clone();
    let settings = &menu.settings;
    let unset = i18n::text(&ctx, "boot-sdboot-unset").into_owned();
    let switch = |value: Option<bool>| match value {
        Some(true) => i18n::text(&ctx, "boot-sdboot-on").into_owned(),
        Some(false) => i18n::text(&ctx, "boot-sdboot-off").into_owned(),
        None => unset.clone(),
    };

    let default = default_text(&ctx, menu);
    let timeout = settings
        .timeout
        .map_or_else(|| unset.clone(), |timeout| timeout_text(&ctx, timeout));

    fact(
        ui,
        true,
        "boot-default",
        "boot-default-known",
        Some(&default),
    );
    fact(
        ui,
        false,
        "boot-timeout",
        "boot-timeout-detail",
        Some(&timeout),
    );
    fact(
        ui,
        false,
        "boot-sdboot-editor",
        "boot-sdboot-editor-detail",
        Some(&switch(settings.editor)),
    );
    fact(
        ui,
        false,
        "boot-sdboot-auto-entries",
        "boot-sdboot-auto-entries-detail",
        Some(&switch(settings.auto_entries)),
    );
    fact(
        ui,
        false,
        "boot-sdboot-auto-firmware",
        "boot-sdboot-auto-firmware-detail",
        Some(&switch(settings.auto_firmware)),
    );
}

fn default_text(ctx: &egui::Context, menu: &SdBootMenu) -> String {
    menu.settings.default.as_ref().map_or_else(
        || i18n::text(ctx, "boot-sdboot-default-first").into_owned(),
        |default| match default.as_str() {
            "@saved" => i18n::text(ctx, "boot-default-last").into_owned(),
            pattern => menu
                .choosable()
                .find(|entry| entry.id.as_str() == pattern)
                .map_or_else(|| pattern.to_owned(), |entry| entry.title.clone()),
        },
    )
}

fn timeout_text(ctx: &egui::Context, timeout: LoaderTimeout) -> String {
    match timeout {
        LoaderTimeout::Seconds(seconds) => {
            format!("{seconds} {}", i18n::text(ctx, "boot-timeout-unit"))
        }
        word @ (LoaderTimeout::MenuForce
        | LoaderTimeout::MenuHidden
        | LoaderTimeout::MenuDisabled) => {
            i18n::text(ctx, &format!("boot-sdboot-timeout-{word}")).into_owned()
        }
    }
}

/// Phase one shows the settings without changing them, so the value is the
/// row's state line instead of a control.
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
