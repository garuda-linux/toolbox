use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{Label, Ui};
use toolbox_catalog::{Check, ToggleEntry};
use toolbox_core::Toggle;
use toolbox_core::features::toggles::shown::Entry;
use toolbox_gui_kit::icons;
use toolbox_gui_kit::widgets::{
    RowState, Tone, dim_picture, icon_button, icon_button_width, paint_letter_tile, queued_tag,
    row_context_menu, row_height, split_row, switch, text_block_height,
};

use super::catalog_text;
use super::listed::{EntryRow, EntryRows};
use crate::catalogs::{self, CatalogId};
use crate::command::AppCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::{
    Initial, PICTURE_RADIUS, SWITCH_WIDTH, actions, picture_height, picture_or_initial,
    picture_slot,
};

#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub catalog: CatalogId,
    /// A plan runs, so the switch rests.
    pub busy: bool,
}

/// A row's height before it was laid out: the title and one line of
/// description. Longer descriptions wrap and grow it.
pub fn estimate(ui: &Ui) -> f32 {
    row_height(ui, text_block_height(ui, 1).max(picture_height(ui, true)))
}

pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    place: Place,
    row: &EntryRow,
    state: &RowState,
) {
    let width = SWITCH_WIDTH + row.url.as_ref().map_or(0.0, |_| icon_button_width(ui));

    split_row(
        ui,
        width,
        |ui| {
            ui.horizontal_top(|ui| {
                let at = picture_slot(ui, true);
                ui.vertical(|ui| text(ui, row));

                // Its title is dim when the row can't change.
                dim_picture(ui, at, !row.changeable, |ui| {
                    if let Some(picture) = &row.picture {
                        let initial = Initial {
                            shown: &row.title,
                            package: &row.tile_package,
                        };

                        _ = picture_or_initial(ui, picture, at, initial);
                    } else {
                        paint_letter_tile(ui, at, PICTURE_RADIUS, &row.title, &row.tile_package);
                    }
                });
            });
        },
        |ui| {
            let enabled = row.changeable && !place.busy;
            let response = switch(ui, row.on, &row.title, enabled);

            // The page banner says it once to the eye, a screen reader
            // hears it on the row it reads.
            if row.unknown {
                let unknown = i18n::text(ui.ctx(), "toggles-state-unknown").into_owned();
                ui.ctx()
                    .accesskit_node_builder(response.id, |node| node.set_description(unknown));
            }

            let flipped = response.clicked();
            if flipped || (state.toggle && enabled) {
                context.commands.send(AppCommand::ToggleEntry {
                    catalog: place.catalog,
                    entry: row.id.clone(),
                    wish: if row.on { Toggle::Off } else { Toggle::On },
                });
            }

            if let (Some(url), Some(name)) = (&row.url, &row.website)
                && icon_button(ui, icons::GLOBE, name).clicked()
            {
                crate::links::open(ui.ctx(), url);
            }
        },
    );

    row_context_menu(ui, state, &row.menu, |ui| {
        actions::menu::<EntryRows>(ui, row, context.commands);
    });
}

fn text(ui: &mut Ui, row: &EntryRow) {
    toolbox_gui_kit::widgets::title_line(ui, true, |ui| {
        ui.label(Arc::clone(&row.title_job));
        if let Some((note, tone)) = &row.note {
            queued_tag(ui, note, *tone);
        }
    });

    ui.add(Label::new(Arc::clone(&row.description)).wrap());
}

/// Whether `query` (trimmed, lower case, empty for all) is in the entry's
/// title or description, as the user reads them.
pub fn matches(ctx: &egui::Context, id: CatalogId, entry: &Entry<'_>, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let title = catalog_text(ctx, id, entry.entry.id(), entry.entry.title());
    title.to_lowercase().contains(query)
        || description(ctx, id, entry).to_lowercase().contains(query)
}

pub fn description(ctx: &egui::Context, id: CatalogId, entry: &Entry<'_>) -> String {
    catalog_text(
        ctx,
        id,
        &format!("{}-description", entry.entry.id()),
        entry.entry.description(),
    )
}

/// What the entry waits for, as a tag beside its title: the apply, or an
/// entry it needs.
pub fn note(
    ctx: &egui::Context,
    id: CatalogId,
    entry: &Entry<'_>,
) -> Option<(std::borrow::Cow<'static, str>, Tone)> {
    if let Some(why) = entry.state.kept {
        let text = i18n::text(ctx, &format!("protected-{}", why.as_str()));
        return Some((text, Tone::Neutral));
    }

    if let Some(wish) = entry.state.queued {
        let text = if wish.is_on() {
            "toggles-turns-on"
        } else {
            "toggles-turns-off"
        };

        return Some((i18n::text(ctx, text), Tone::Accent));
    }

    if entry.changeable() {
        return None;
    }

    let names = entry
        .requires
        .iter()
        .map(|needed| catalog_text(ctx, id, needed.id(), needed.title()))
        .collect::<Vec<_>>()
        .join(", ");

    let text = if entry.requires_any {
        "toggles-needs-any"
    } else {
        "toggles-needs"
    };

    Some((
        i18n::text_with(ctx, text, &[("names", &names)]),
        Tone::Neutral,
    ))
}

/// What picks the hue of the entry's letter tile: the package it checks,
/// so it matches the package's tile on other pages, else its catalog id.
pub fn tile_package(entry: &ToggleEntry) -> &str {
    match entry.check() {
        Check::Package(package) => package.as_str(),
        Check::Service(_) | Check::UserService(_) | Check::Group(_) | Check::UserConfig(_) => {
            entry.id()
        }
    }
}

/// The entry's picture: the icon theme's own when it has one, else the
/// one the catalog ships.
pub fn picture_file(
    context: &PageContext<'_>,
    id: CatalogId,
    entry: &Entry<'_>,
) -> Option<PathBuf> {
    let themed = context.app_icons.and_then(|icons| {
        catalogs::icon_names(entry.entry)
            .iter()
            .find_map(|name| icons.get(name).map(Path::to_path_buf))
    });

    themed.or_else(|| {
        entry
            .entry
            .icon()
            .map(|name| context.catalogs.get(id).icon_dir.join(name))
    })
}
