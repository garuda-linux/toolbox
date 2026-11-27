//! The parts under the columns: the installed files and the package's
//! newest lines of pacman's log.

use std::borrow::Cow;
use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Label, RichText, ScrollArea, TextStyle, Ui, WidgetInfo};
use toolbox_core::features::packages::PackageDetails;
use toolbox_core::features::packages::detail::{HISTORY_SHOWN, file_list, matching_files, shown};
use toolbox_core::features::packages::history::{HistoryAction, HistoryEntry};
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::theme::show_scroll_rows;
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_gui_kit::widgets::{
    ButtonKind, PropertyTexts, SearchTexts, counted_disclosure, disclosure, loading_indicator,
    padded_card, pill_button, property_list, push_height, rich, search_field,
};
use toolbox_gui_kit::{UiExt as _, tokens};
use toolbox_text::OutsideText;

use super::super::PackagesPage;
use crate::command::AppCommand;
use crate::i18n::Arg;
use crate::packages::{Details, Read};
use crate::pages::PageContext;
use crate::pages::rows::Look;
use crate::{dates, i18n};

/// The file box is at most this tall, so a package of thousands of files
/// keeps its history in reach.
const FILES_HEIGHT: f32 = 320.0;

const FILTER_WIDTH: f32 = 240.0;

/// The history's lines as the page shows them, and how many Show all
/// adds.
#[derive(Debug, Default)]
pub struct History {
    lines: Vec<(String, String)>,
    hidden: usize,
}

/// Keyed by the details, whether every line shows and the look.
pub type HistoryRows = RowCache<(Same<Details>, bool, Look), History>;

/// The places of the files under the filter, keyed by the details.
pub type FileRows = RowCache<Same<Details>, Vec<usize>>;

/// The files of an installed package: the fold with their count and Copy
/// list, then the filter and one monospaced line per file in a box of
/// its own, only the visible lines laid out.
pub fn files(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    (details, info): (&Arc<Details>, &PackageDetails),
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
        ui.set_min_height(push_height(ui));
        if counted_disclosure(ui, &title, &count, page.unfolded.files).clicked() {
            page.unfolded.files = !page.unfolded.files;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let copy = i18n::text(&ctx, "packages-copy-list");
            let enabled = !installed.files.is_empty();
            let clicked = ui
                .enabled_ui(enabled, |ui| pill_button(ui, &copy, ButtonKind::Soft))
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

    ui.add_space(ui.tokens().sizes.tight_gap);
    let name = i18n::text(&ctx, "packages-files-filter");
    let clear = i18n::text(&ctx, "packages-files-filter-clear");
    let texts = SearchTexts {
        name: &name,
        hint: &name,
        clear: &clear,
    };

    _ = search_field(ui, texts.name, &mut page.file_filter, texts, FILTER_WIDTH);
    let filter = &page.file_filter;
    let places = page.files.get(&[filter.as_str()], Same::of(details), || {
        let _build = tracing::trace_span!("filter files").entered();
        matching_files(&installed.files, filter)
    });

    ui.add_space(ui.tokens().sizes.control_gap);
    if places.is_empty() {
        let tokens = tokens::current(&ctx);
        let none = i18n::text_with(
            &ctx,
            "packages-files-none",
            &[("filter", Arg::from(filter.trim()))],
        );

        ui.add(Label::new(rich(ui, &none, Weight::Regular, BODY, tokens.secondary)).wrap());

        return;
    }

    file_lines(ui, &installed.files, &places);
}

/// The lines of `places` in `files`, in a box that scrolls on its own.
fn file_lines(ui: &mut Ui, files: &[OutsideText], places: &[usize]) {
    let tokens = tokens::current(ui.ctx());
    let font = TextStyle::Monospace.resolve(ui.style());
    let height = ui.fonts_mut(|fonts| fonts.row_height(&font));
    let label = i18n::text(ui.ctx(), "packages-files-list");

    padded_card(ui, |ui| {
        // show_rows counts the spacing of this ui between rows.
        ui.spacing_mut().item_spacing.y = 0.0;
        show_scroll_rows(
            ui,
            ScrollArea::vertical()
                .id_salt("packages-files")
                .auto_shrink([false, true])
                .max_height(FILES_HEIGHT),
            height,
            places.len(),
            |ui, shown| {
                let lines = places
                    .get(shown)
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|&place| files.get(place));

                for file in lines {
                    let text = RichText::new(file.as_str())
                        .font(font.clone())
                        .color(tokens.text);

                    ui.add(Label::new(text).truncate());
                }
            },
        );
    })
    .response
    .widget_info(|| WidgetInfo::labeled(Role::Document, true, &*label));
}

pub fn history(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    details: &Arc<Details>,
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

    let entries = match &details.history {
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

    let every_line = page.unfolded.history_all;
    let rest = (Same::of(details), every_line, Look::of(ui));
    let built = page.history.get(&[], rest, || {
        let _build = tracing::trace_span!("build rows").entered();
        let part = shown(entries, HISTORY_SHOWN, every_line);
        History {
            lines: part.items.iter().map(|entry| line(&ctx, entry)).collect(),
            hidden: part.hidden,
        }
    });

    let copy = |label: &str| label.to_owned();
    let texts = PropertyTexts {
        copy: &copy,
        copied: "",
    };

    ui.add_space(ui.tokens().sizes.tight_gap);
    padded_card(ui, |ui| {
        property_list(ui, texts, |list| {
            for (when, what) in &built.lines {
                list.row(when, what);
            }
        });
    });

    if built.hidden > 0 {
        ui.add_space(ui.tokens().sizes.card_gap);
        let all = i18n::text(&ctx, "packages-history-all");
        if pill_button(ui, &all, ButtonKind::Soft).clicked() {
            page.unfolded.history_all = true;
        }
    }
}

/// The moment of a change in the locale's format, and the change.
fn line(ctx: &egui::Context, entry: &HistoryEntry) -> (String, String) {
    let when = dates::pacman_moment(ctx, &entry.when).unwrap_or_else(|| entry.when.clone());
    (when, change(ctx, entry))
}

/// The ids are literals, so grep and the catalog checks see each one.
const fn action_id(action: HistoryAction) -> &'static str {
    match action {
        HistoryAction::Installed => "packages-history-installed",
        HistoryAction::Upgraded => "packages-history-upgraded",
        HistoryAction::Downgraded => "packages-history-downgraded",
        HistoryAction::Reinstalled => "packages-history-reinstalled",
        HistoryAction::Removed => "packages-history-removed",
    }
}

/// "Upgraded 3.0.2-1 → 3.0.4-1".
fn change(ctx: &egui::Context, entry: &HistoryEntry) -> String {
    let action = i18n::text(ctx, action_id(entry.action));
    let versions: Cow<'_, str> = if entry.versions.contains(" -> ") {
        Cow::Owned(entry.versions.replace(" -> ", " → "))
    } else {
        Cow::Borrowed(&entry.versions)
    };

    format!("{action} {versions}")
}
