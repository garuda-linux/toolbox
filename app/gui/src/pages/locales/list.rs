use std::sync::Arc;

use egui::{Id, Label, Ui};
use toolbox_core::features::locales::view::locales;
use toolbox_core::features::locales::{Filter, pick};
use toolbox_core::features::toggles::shown::Control;
use toolbox_gui_kit::fonts::{self, BODY, SMALL, Weight};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{
    ListItem, RowHeights, RowList, RowState, Tone, highlighted, rich, row_height, split_row_in,
    status_dot, text_block_height,
};
use toolbox_plan::LocaleEntry;

use crate::command::LocaleCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::rows::Look;
use crate::pages::toggles::control::{self, Setting};

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
    let ctx = ui.ctx();
    let rest = (
        filter.utf8_only,
        filter.enabled_only,
        Same::of(context.snapshot),
        context.revision,
        Look::of(ui),
    );

    cache.get(&[&filter.search], rest, || {
        let rows = locales(context.snapshot, context.changes);
        pick(&rows, filter)
            .into_iter()
            .filter_map(|at| rows.get(at))
            .map(|row| {
                let code = row.entry.locale.as_str();
                let language = row.name.as_ref().map_or("", |name| name.language.as_str());
                let territory = row.name.as_ref().map_or("", |name| name.territory.as_str());
                let title = if language.is_empty() {
                    code.to_owned()
                } else if territory.is_empty() {
                    language.to_owned()
                } else {
                    let names = [
                        ("language", Arg::from(language)),
                        ("territory", Arg::from(territory)),
                    ];

                    i18n::text_with(ctx, "locales-title-territory", &names).into_owned()
                };

                // A code such as en_US.UTF-8 already names its charset.
                let charset = row
                    .unnamed_charset()
                    .filter(|charset| !code.contains(charset));

                let detail_id = match (charset.is_some(), row.system) {
                    (true, true) => "locales-detail-system",
                    (true, false) => "locales-detail",
                    (false, true) => "locales-detail-code-system",
                    (false, false) => "locales-detail-code",
                };

                let detail = i18n::text_with(
                    ctx,
                    detail_id,
                    &[
                        ("code", Arg::from(code)),
                        ("charset", Arg::from(charset.unwrap_or_default())),
                    ],
                );

                LocaleItem {
                    key: row.text.clone(),
                    title,
                    detail: detail.into_owned(),
                    enabled: row.enabled,
                    on: row.is_on(),
                    queued: row.queued.is_some(),
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
    };

    // A locale is a setting, but this page also lists packages, so a
    // setting has the Enable and Disable buttons of a mixed page.
    let shown = control::Shown {
        control: Control::Switch,
        on: item.on,
        title: &item.title,
        enabled: true,
        unknown: false,
        setting: Setting::Button,
    };

    let width = control::button_width(ui);
    if line_row(ui, state.row, &line, width, |ui| {
        control::show(ui, shown, state)
    }) {
        context.commands.send(LocaleCommand::Toggle {
            entry: item.entry.clone(),
            on: toolbox_core::Toggle::from_on(!item.on),
        });
    }
}

/// A line with its words and the control `right` draws at the end.
pub(super) fn line_row(
    ui: &mut Ui,
    row: Id,
    line: &Line<'_>,
    width: f32,
    right: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    let ctx = ui.ctx().clone();
    let mut picked = false;
    split_row_in(
        ui,
        row,
        width,
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

            // The cut is the catalog rows' own, and the tooltip keeps the whole.
            let font = fonts::font(ui.style(), Weight::Regular, SMALL);
            let cut = control::cut(highlighted(ui, line.detail, "", font, tokens.secondary));
            ui.add(Label::new(cut).wrap()).on_hover_text(line.detail);
        },
        |ui| {
            // The list waits in a disabled Ui while a plan runs.
            picked = right(ui);
        },
    );

    picked
}
