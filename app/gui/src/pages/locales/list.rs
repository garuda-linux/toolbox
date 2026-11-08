use std::sync::Arc;

use egui::{Label, Ui};
use toolbox_core::features::locales::view::locales;
use toolbox_core::features::locales::{Filter, pick};
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{
    ListItem, RowHeights, RowList, RowState, Tone, rich, row_height, split_row, status_dot, switch,
    text_block_height,
};
use toolbox_plan::LocaleEntry;

use crate::command::LocaleCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::{Look, SWITCH_WIDTH};

/// One locale as the list shows it, its texts made once.
#[derive(Debug)]
pub struct LocaleItem {
    /// The `locale.gen` line, unique in the list.
    key: String,
    title: String,
    detail: String,
    enabled: bool,
    on: bool,
    queued: bool,
    entry: LocaleEntry,
}

pub struct Line<'a> {
    pub title: &'a str,
    pub detail: &'a str,
    pub status: (&'static str, Tone),
    pub queued: bool,
    pub on: bool,
}

pub type Rows = RowCache<(bool, bool, Same<toolbox_core::Snapshot>, u64, Look), Vec<LocaleItem>>;

impl ListItem for LocaleItem {
    fn key(&self) -> &str {
        &self.key
    }

    fn name(&self) -> &str {
        &self.title
    }
}

pub fn rows(
    ui: &Ui,
    context: &PageContext<'_>,
    filter: &Filter,
    cache: &mut Rows,
) -> Arc<Vec<LocaleItem>> {
    let ctx = ui.ctx().clone();
    let rest = (
        filter.utf8_only,
        filter.enabled_only,
        Same::of(context.snapshot),
        context.revision,
        Look::of(ui),
    );

    cache.get(&[&filter.search], rest, || {
        let (rows, shown) = locales(context.snapshot, context.changes);
        pick(&rows, filter)
            .into_iter()
            .filter_map(|at| Some((rows.get(at)?, shown.get(at)?)))
            .map(|(row, shown)| {
                let title = if shown.language.is_empty() {
                    shown.locale_code.clone()
                } else if shown.territory.is_empty() {
                    shown.language.clone()
                } else {
                    i18n::text_with(
                        &ctx,
                        "locales-title-territory",
                        &[
                            ("language", &shown.language),
                            ("territory", &shown.territory),
                        ],
                    )
                    .into_owned()
                };

                let detail_id = if shown.system {
                    "locales-detail-system"
                } else {
                    "locales-detail"
                };

                let detail = i18n::text_with(
                    &ctx,
                    detail_id,
                    &[("code", &shown.locale_code), ("charset", &shown.charset)],
                );

                LocaleItem {
                    key: row.text.clone(),
                    title,
                    detail: detail.into_owned(),
                    enabled: shown.enabled_now,
                    on: shown.on,
                    queued: !shown.queued.is_empty(),
                    entry: row.entry.clone(),
                }
            })
            .collect()
    })
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, items: &[LocaleItem]) {
    let name = i18n::text(ui.ctx(), "locales-list");
    let height = row_height(ui, text_block_height(ui, 1));
    RowList::new("locales", &name, RowHeights::Fixed(height))
        .show(ui, items, |ui, item, state| row(ui, context, item, state));
}

fn row(ui: &mut Ui, context: &PageContext<'_>, item: &LocaleItem, state: &RowState) {
    let status = if item.enabled {
        ("locales-enabled", Tone::Success)
    } else {
        ("locales-disabled", Tone::Neutral)
    };

    let line = Line {
        title: &item.title,
        detail: &item.detail,
        status,
        queued: item.queued,
        on: item.on,
    };

    if switch_line(ui, context, &line, state) {
        context.commands.send(LocaleCommand::Toggle {
            entry: item.entry.clone(),
            on: !item.on,
        });
    }
}

pub fn switch_line(
    ui: &mut Ui,
    context: &PageContext<'_>,
    line: &Line<'_>,
    state: &RowState,
) -> bool {
    let ctx = ui.ctx().clone();
    let mut flipped = false;
    split_row(
        ui,
        SWITCH_WIDTH,
        |ui| {
            let tokens = tokens::current(&ctx);
            toolbox_gui_kit::widgets::title_line(ui, false, |ui| {
                ui.add(
                    Label::new(rich(ui, line.title, Weight::SemiBold, BODY, tokens.text))
                        .truncate(),
                );
                let (word, tone) = line.status;
                status_dot(ui, &i18n::text(&ctx, word), tone);
                if line.queued {
                    status_dot(ui, &i18n::text(&ctx, "locales-queued"), Tone::Accent);
                }
            });

            ui.add(
                Label::new(rich(
                    ui,
                    line.detail,
                    Weight::Regular,
                    SMALL,
                    tokens.secondary,
                ))
                .truncate(),
            );
        },
        |ui| {
            let enabled = !context.busy;
            let clicked = switch(ui, line.on, line.title, enabled).clicked();
            flipped = clicked || (state.toggle && enabled);
        },
    );

    flipped
}
