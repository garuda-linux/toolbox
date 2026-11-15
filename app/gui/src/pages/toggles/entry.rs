use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{CornerRadius, Label, Rect, Ui, Vec2};
use toolbox_catalog::{Check, ToggleEntry};
use toolbox_core::Toggle;
use toolbox_core::features::toggles::shown::{Control, Entry};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{
    RowState, Tone, dim_picture, page_tile, paint_letter_tile, queued_tag, row_height, row_menu,
    split_row, status_dot, text_block_height,
};

use super::catalog_text;
use super::control::{self, Setting};
use super::listed::{EntryRow, EntryRows};
use crate::artwork::Artwork;
use crate::catalogs::{self, CatalogId};
use crate::command::AppCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::{
    Initial, PICTURE_RADIUS, actions, picture_height, picture_or_initial, picture_slot,
};

/// The share of an icon tile's edge its glyph takes, as in gui-kit.
const GLYPH_SHARE: f32 = 0.56;

#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub catalog: CatalogId,
    /// A plan runs, so the control rests.
    pub busy: bool,
}

/// A row's height before it was laid out: the title and one line of
/// description. A description of two lines grows it.
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
    let setting = if place.catalog == CatalogId::Settings {
        Setting::Switch
    } else {
        Setting::Button
    };

    let width = control::width(ui, row.control, setting);

    split_row(
        ui,
        width,
        |ui| {
            ui.horizontal_top(|ui| {
                let at = picture_slot(ui, true);
                ui.vertical(|ui| text(ui, row));

                dim_picture(ui, at, !row.changeable, |ui| {
                    if let Some(glyph) = row.glyph {
                        paint_glyph(ui, at, glyph);
                    } else if let Some(picture) = &row.picture {
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
            // `control::width` reserves exactly menu, gap and control. egui's
            // item spacing on top would overflow the slot and widen the card
            // row after row.
            ui.spacing_mut().item_spacing.x = 0.0;

            // The website and the name are no change, so only the
            // control rests while a plan runs.
            row_menu(ui, state, &row.menu, |ui| {
                actions::menu::<EntryRows>(ui, row, context.commands);
            });
            ui.add_space(ui.tokens().sizes.control_gap);

            let shown = control::Shown {
                control: row.control,
                on: row.on,
                title: &row.title,
                enabled: row.changeable && !place.busy,
                unknown: row.unknown,
                setting,
            };

            if control::show(ui, shown, state) {
                context.commands.send(AppCommand::ToggleEntry {
                    catalog: place.catalog,
                    entry: row.id.clone(),
                    wish: if row.on { Toggle::Off } else { Toggle::On },
                });
            }
        },
    );
}

/// The glyph on the page's tile. It fills the picture slot rather than an
/// icon tile's edge, so the row lines up with the rows that have pictures.
pub fn paint_glyph(ui: &Ui, at: Rect, glyph: Icon) {
    let tile = page_tile(ui.ctx());
    ui.painter()
        .rect_filled(at, CornerRadius::same(PICTURE_RADIUS), tile.fill);

    let theme = icons::theme_in_use(ui.ctx());
    let edge = at.width() * GLYPH_SHARE;
    glyph
        .image(theme.as_deref(), tile.ink)
        .paint_at(ui, Rect::from_center_size(at.center(), Vec2::splat(edge)));
}

fn text(ui: &mut Ui, row: &EntryRow) {
    toolbox_gui_kit::widgets::title_line(ui, true, |ui| {
        ui.label(Arc::clone(&row.title_job));
        if let Some((note, tone)) = &row.note {
            queued_tag(ui, note, *tone);
        }

        if let Some(installed) = &row.installed {
            status_dot(ui, installed, Tone::Success);
        }
    });

    ui.add(Label::new(Arc::clone(&row.description)));
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
/// entry it needs. A package installs or removes, a setting turns on or
/// off.
pub fn note(
    ctx: &egui::Context,
    id: CatalogId,
    entry: &Entry<'_>,
    kind: Control,
) -> Option<(std::borrow::Cow<'static, str>, Tone)> {
    if let Some(why) = entry.state.kept {
        let text = i18n::text(ctx, &format!("protected-{}", why.as_str()));
        return Some((text, Tone::Neutral));
    }

    if let Some(wish) = entry.state.queued {
        let text = control::queued_text(wish, kind);
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
    let themed = context.service::<Artwork>().app_icons().and_then(|icons| {
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
