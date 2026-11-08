use std::sync::Arc;

use egui::Ui;
use toolbox_core::features::locales::view::{pack_rows, packs};
use toolbox_core::features::locales::{PackOrder, pack_order};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    ListItem, RowHeights, RowList, RowState, Tone, row_height, text_block_height,
};
use toolbox_plan::PackageName;

use super::list::{Line, switch_line};
use crate::command::LocaleCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::Look;

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

pub type Rows = RowCache<(PackOrder, Same<toolbox_core::Snapshot>, u64, Look), Vec<PackItem>>;

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
    order: PackOrder,
    cache: &mut Rows,
) -> Arc<Vec<PackItem>> {
    let ctx = ui.ctx().clone();
    let rest = (
        order,
        Same::of(context.snapshot),
        context.revision,
        Look::of(ui),
    );

    cache.get(&[], rest, || {
        let rows = pack_rows(context.snapshot, context.changes);
        let shown = packs(&rows);
        pack_order(&rows, order)
            .into_iter()
            .filter_map(|at| Some((rows.get(at)?, shown.get(at)?)))
            .map(|(row, shown)| {
                let locale =
                    super::choice(&ctx, &shown.locale_code, &shown.language, &shown.territory);

                let detail = i18n::text_with(
                    &ctx,
                    "locales-pack-detail",
                    &[("base", &shown.base), ("locale", &locale)],
                );

                PackItem {
                    title: shown.pack.clone(),
                    detail: detail.into_owned(),
                    installed: shown.installed,
                    on: shown.on,
                    queued: !shown.queued.is_empty(),
                    package: row.pack.package.clone(),
                }
            })
            .collect()
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
        on: item.on,
    };

    if switch_line(ui, context, &line, state) {
        context.commands.send(LocaleCommand::TogglePack {
            package: item.package.clone(),
            on: !item.on,
        });
    }
}
