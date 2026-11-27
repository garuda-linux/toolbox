use std::sync::Arc;

use egui::Ui;
use toolbox_core::features::locales::view::{pack_rows, packs, packs_missing};
use toolbox_core::features::locales::{PackOrder, pack_pick};
use toolbox_core::features::toggles::shown::Control;
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    ListItem, RowHeights, RowList, RowState, Tone, row_height, text_block_height,
};
use toolbox_plan::PackageName;

use super::list::{Line, line_row};
use crate::command::LocaleCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::rows::Look;
use crate::pages::toggles::control::{self, Setting};

/// One pack as the list shows it, its texts made once.
#[derive(Debug)]
pub struct PackItem {
    title: String,
    detail: String,
    installed: bool,
    on: bool,
    queued: bool,
    package: PackageName,
}

/// The packs the search finds, and how many of all packs are missing.
#[derive(Debug)]
pub struct Packs {
    pub items: Vec<PackItem>,
    /// Packs before the search, so the search tells no match from none.
    pub all: usize,
    pub missing: usize,
}

pub type Rows = RowCache<(PackOrder, Same<toolbox_core::Snapshot>, u64, Look), Packs>;

impl ListItem for PackItem {
    fn key(&self) -> &str {
        &self.title
    }

    fn name(&self) -> &str {
        &self.title
    }
}

pub fn rows(
    ui: &Ui,
    context: &PageContext<'_>,
    (order, search): (PackOrder, &str),
    cache: &mut Rows,
) -> Arc<Packs> {
    let ctx = ui.ctx();
    let rest = (
        order,
        Same::of(context.snapshot),
        context.revision,
        Look::of(ui),
    );

    cache.get(&[search], rest, || {
        let rows = pack_rows(context.snapshot, context.changes);
        let shown = packs(&rows);
        let items = pack_pick(&rows, order, search)
            .into_iter()
            .filter_map(|at| rows.get(at).zip(shown.get(at)))
            .map(|(row, shown)| {
                let locale = super::system::choice(
                    ctx,
                    &shown.locale_code,
                    shown.language.as_deref(),
                    shown.territory.as_deref(),
                );

                let detail = i18n::text_with(
                    ctx,
                    "locales-pack-detail",
                    &[
                        ("base", Arg::from(&shown.base)),
                        ("locale", Arg::from(&locale)),
                    ],
                );

                PackItem {
                    title: shown.pack.clone(),
                    detail: detail.into_owned(),
                    installed: shown.installed,
                    on: shown.on,
                    queued: shown.queued.is_some(),
                    package: row.pack.package.clone(),
                }
            })
            .collect();

        Packs {
            items,
            all: rows.len(),
            missing: packs_missing(&rows),
        }
    })
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, items: &[PackItem]) {
    let name = i18n::text(ui.ctx(), "locales-packs-list");
    let height = row_height(ui, text_block_height(ui, 1));
    RowList::new("language-packs", &name, RowHeights::Fixed(height)).show(
        ui,
        items,
        |ui, item, state| row(ui, context, item, state),
    );
}

fn row(ui: &mut Ui, context: &PageContext<'_>, item: &PackItem, state: &RowState) {
    let status = if item.installed {
        ("locales-pack-installed", Tone::Success)
    } else {
        ("locales-pack-missing", Tone::Neutral)
    };

    let line = Line {
        title: &item.title,
        detail: &item.detail,
        status,
        queued: item.queued,
    };

    // A pack is a package, so it has the package buttons, not a switch.
    let control = if item.on {
        Control::Remove
    } else {
        Control::Install
    };

    let shown = control::Shown {
        control,
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
        context.commands.send(LocaleCommand::TogglePack {
            package: item.package.clone(),
            on: toolbox_core::Toggle::from_on(!item.on),
        });
    }
}
