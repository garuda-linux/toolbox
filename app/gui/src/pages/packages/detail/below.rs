//! The parts under the columns: the installed files, folded at first,
//! with Copy list, and the package's newest lines of pacman's log, open
//! at first, with Show all.

use std::borrow::Cow;

use egui::{Label, Ui};
use toolbox_core::features::packages::PackageDetails;
use toolbox_core::features::packages::detail::{HISTORY_SHOWN, file_list, shown};
use toolbox_core::features::packages::history::HistoryEntry;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::theme::PILL_HEIGHT;
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, PropertyTexts, RowHeights, RowList, counted_disclosure, disclosure,
    loading_indicator, pill_button, property_list, rich, row_height, text_block_height,
};
use toolbox_gui_kit::{UiExt as _, tokens};

use super::super::PackagesPage;
use crate::command::AppCommand;
use crate::i18n;
use crate::packages::Read;
use crate::pages::PageContext;

/// One installed file, keyed by its path.
struct Line<'a> {
    text: &'a str,
}

impl ListItem for Line<'_> {
    fn key(&self) -> &str {
        self.text
    }

    fn name(&self) -> &str {
        self.text
    }
}

/// The files of an installed package: the fold with their count and Copy
/// list, then one line per file, only the visible ones laid out.
pub fn files(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    info: &PackageDetails,
) {
    let Some(installed) = &info.installed else {
        return;
    };

    let ctx = ui.ctx().clone();
    let count = installed.files.len().to_string();
    let title = i18n::text(&ctx, "packages-files");

    ui.horizontal(|ui| {
        // As tall as Copy list, so the fold centres on the button's middle
        // and not on egui's shorter row.
        ui.set_min_height(PILL_HEIGHT);
        if counted_disclosure(ui, &title, &count, page.unfolded.files).clicked() {
            page.unfolded.files = !page.unfolded.files;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let copy = i18n::text(&ctx, "packages-copy-list");
            let enabled = !installed.files.is_empty();
            let clicked = ui
                .add_enabled_ui(enabled, |ui| pill_button(ui, &copy, ButtonKind::Soft))
                .inner
                .clicked();

            if clicked {
                ctx.copy_text(file_list(&installed.files));
                context.commands.send(AppCommand::Toast {
                    kind: ToastKind::Info,
                    text: i18n::text(&ctx, "packages-files-copied").into_owned(),
                });
            }
        });
    });

    if !page.unfolded.files || installed.files.is_empty() {
        return;
    }

    let items: Vec<Line<'_>> = installed
        .files
        .iter()
        .map(|file| Line { text: file })
        .collect();

    let name = i18n::text(&ctx, "packages-files-list");
    let height = row_height(ui, text_block_height(ui, 0));
    RowList::new("packages-files-list", &name, RowHeights::Fixed(height)).show(
        ui,
        &items,
        |ui, line, _| {
            let tokens = tokens::current(ui.ctx());
            ui.add(Label::new(rich(ui, line.text, Weight::Regular, SMALL, tokens.text)).truncate());
        },
    );
}

/// The fold of the history, then its newest lines and Show all.
pub fn history(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    history: &Read<Vec<HistoryEntry>>,
) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, "packages-history");
    let open = !page.unfolded.history_folded;
    if disclosure(ui, &title, open).clicked() {
        page.unfolded.history_folded = open;
    }

    if !open {
        return;
    }

    let entries = match history {
        Read::Known(entries) => entries,
        Read::Loading => {
            loading_indicator(ui, &i18n::text(&ctx, "packages-details-loading"));
            return;
        }
        Read::Unavailable | Read::Failed => {
            super::super::list::unreadable(ui, context, "packages-history-failed");
            return;
        }
    };

    if entries.is_empty() {
        let tokens = tokens::current(&ctx);
        let none = i18n::text(&ctx, "packages-history-none");
        ui.add(Label::new(rich(ui, &none, Weight::Regular, BODY, tokens.secondary)).wrap());
        return;
    }

    let part = shown(entries, HISTORY_SHOWN, page.unfolded.history_all);
    let lines: Vec<(String, String)> = part
        .items
        .iter()
        .map(|entry| (entry.shown_when(), change(&ctx, entry)))
        .collect();

    let copy = |label: &str| label.to_owned();
    let texts = PropertyTexts {
        copy: &copy,
        copied: "",
    };

    ui.scope(|ui| {
        property_list(ui, texts, |list| {
            for (when, what) in &lines {
                list.row(when, what);
            }
        });
    });

    if part.hidden > 0 {
        ui.add_space(ui.tokens().sizes.card_gap);
        let all = i18n::text(&ctx, "packages-history-all");
        if pill_button(ui, &all, ButtonKind::Soft).clicked() {
            page.unfolded.history_all = true;
        }
    }
}

/// "Upgraded 3.0.2-1 → 3.0.4-1".
fn change(ctx: &egui::Context, entry: &HistoryEntry) -> String {
    let action = i18n::text(ctx, &format!("packages-history-{}", entry.action.as_str()));
    let versions: Cow<'_, str> = if entry.versions.contains(" -> ") {
        Cow::Owned(entry.versions.replace(" -> ", " → "))
    } else {
        Cow::Borrowed(&entry.versions)
    };

    format!("{action} {versions}")
}
