//! The all clear: which checks passed, so "nothing to do" says what was
//! looked at.

use egui::Ui;
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::{ActionRow, card, card_row};
use toolbox_services::home::{Check, Overview};

use crate::i18n;
use crate::i18n::Arg;

/// The ids are literals, so grep and the catalog checks see each one.
/// `None` for a check that never passes alone, as the last update's
/// reading only ever fails.
pub(super) const fn check_id(check: Check) -> Option<&'static str> {
    match check {
        Check::Updates => Some("home-check-updates"),
        Check::Aur => Some("home-check-aur"),
        Check::Merges => Some("home-check-merges"),
        Check::LastUpdate => None,
        Check::Disk => Some("home-check-disk"),
        Check::Services => Some("home-check-services"),
        Check::Cleanup => Some("home-check-cleanup"),
        Check::Health => Some("home-check-health"),
    }
}

pub(super) fn all_clear(ui: &mut Ui, overview: &Overview<'_>) {
    let ctx = ui.ctx().clone();
    let count = overview.ran.len();
    let title = i18n::text_with(&ctx, "home-all-clear", &[("count", Arg::from(count))]);
    let names: Vec<String> = overview
        .ran
        .iter()
        .filter_map(|check| check_id(*check))
        .map(|id| i18n::text(&ctx, id).into_owned())
        .collect();

    // The separator is a list's, the same in every language the app has.
    let detail = names.join(" · ");

    card(ui, |ui| {
        card_row(ui, true, |ui| {
            ActionRow::new(icons::DIALOG_POSITIVE, &title)
                .detail(&detail)
                .show(ui, |_| {});
        });
    });
}
