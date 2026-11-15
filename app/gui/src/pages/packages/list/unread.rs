//! What the list shows while the index or the updates are read, and
//! when they could not be.

use egui::Ui;
use toolbox_gui_kit::widgets::{
    SkeletonRows, Tone, banner, card, empty_state_action, skeleton_list,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::super::{PackagesPage, row};
use super::{AurChip, ChipCounts, filters};
use crate::command::PackagesCommand;
use crate::i18n;
use crate::packages::{Packages, Read};
use crate::pages::PageContext;

/// Placeholder rows while the list comes, enough to fill a window.
const SKELETON_ROWS: usize = 6;

/// The chips rest until the index comes, as their counts and the AUR's
/// note need it.
pub(super) fn waiting(page: &mut PackagesPage, ui: &mut Ui, aur_on: bool) {
    let counts = ChipCounts {
        repository: None,
        updates_known: false,
        aur: aur_on.then_some(AurChip { count: None }),
    };

    ui.add_enabled_ui(false, |ui| filters(page, ui, counts));
    ui.add_space(ui.tokens().sizes.header_gap);
    placeholder(ui, "packages-loading");
}

/// Package rows that wait, named by `id`.
pub(super) fn placeholder(ui: &mut Ui, id: &str) {
    let name = i18n::text(ui.ctx(), id);
    let rows = SkeletonRows {
        count: SKELETON_ROWS,
        row_height: row::height(ui),
        leading: true,
    };

    card(ui, |ui| skeleton_list(ui, rows, &name));
}

fn index_text(context: &PageContext<'_>) -> &'static str {
    if matches!(context.service::<Packages>().index(), Read::Unavailable) {
        "packages-unavailable"
    } else {
        "packages-failed"
    }
}

/// Nothing else shows without the index, so its failure fills the page.
pub(super) fn unreadable_page(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let said = i18n::text(&ctx, index_text(context));
    let again = i18n::text(&ctx, "common-read-again");
    if empty_state_action(ui, icons::DIALOG_ERROR, &said, "", &again) {
        context.commands.send(PackagesCommand::Refresh);
    }
}

pub(super) fn unreadable_index(ui: &mut Ui, context: &PageContext<'_>) {
    unreadable(ui, context, index_text(context));
}

pub fn unreadable(ui: &mut Ui, context: &PageContext<'_>, id: &str) {
    let ctx = ui.ctx().clone();
    let again = i18n::text(&ctx, "common-read-again");
    if banner(ui, Tone::Warning, &i18n::text(&ctx, id), &[&again]).is_some() {
        context.commands.send(PackagesCommand::Refresh);
    }
}
