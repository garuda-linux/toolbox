//! The all clear: which checks passed, so "nothing to do" says what was
//! looked at.

use egui::Ui;
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::{ActionRow, Tone, card, card_row};

use super::super::view::Overview;
use crate::i18n;

pub(super) fn all_clear(ui: &mut Ui, overview: &Overview<'_>) {
    let ctx = ui.ctx().clone();
    let count = overview.ran.len().to_string();
    let title = i18n::text_with(&ctx, "home-all-clear", &[("count", &count)]);
    let names: Vec<String> = overview
        .ran
        .iter()
        .map(|check| i18n::text(&ctx, &format!("home-check-{}", check.as_str())).into_owned())
        .collect();

    // The separator is a list's, the same in every language the app has.
    let detail = names.join(" · ");

    card(ui, |ui| {
        card_row(ui, true, |ui| {
            ActionRow::new(icons::DIALOG_POSITIVE, Tone::Success, &title)
                .detail(&detail)
                .show(ui, |_| {});
        });
    });
}
