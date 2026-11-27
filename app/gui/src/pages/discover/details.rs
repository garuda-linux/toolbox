//! The details of one app beside its list: its picture, name and
//! description, its state with the button that installs or removes it, the
//! packages it installs with their repositories and its website.

use egui::{Label, Ui};
use toolbox_core::features::discover::view::Shown;
use toolbox_core::features::discover::{Apps, Hit, package};
use toolbox_core::features::packages::Origin;
use toolbox_core::features::packages::list::Catalog;
use toolbox_core::features::toggles;
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::row_cache::Same;
use toolbox_gui_kit::widgets::{
    ButtonKind, minor_heading, padded_card, pill_button_with_icon, rich, section_heading,
    status_dot, text_link,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
use toolbox_plan::PackageName;

use super::app_row::AppRow;
use super::{DiscoverPage, row};
use crate::i18n;
use crate::i18n::Arg;
use crate::packages::Packages;
use crate::pages::PageContext;
use crate::pages::rows::{Look, picture_slot};
use crate::pages::toggles::control;

/// The app the details show, as its row shows it, the whole description
/// (the row cuts it) and the packages the app installs.
#[derive(Debug)]
pub(super) struct Detail {
    pub row: AppRow,
    pub text: String,
    pub packages: Vec<PackageName>,
}

/// The details of the app `id`, built when the app or the queue changes.
pub fn build(ui: &Ui, context: &PageContext<'_>, id: &str) -> Option<Detail> {
    let hit = find(&context.catalogs.apps.apps, id)?;
    let state = toggles::state(hit.catalog, hit.entry, context.snapshot, context.changes);
    let packages = package(hit.entry)
        .into_iter()
        .chain(hit.entry.also().iter())
        .cloned()
        .collect();

    Some(Detail {
        row: AppRow::of(ui, &Shown { hit, state }, ""),
        text: hit.entry.description().to_owned(),
        packages,
    })
}

fn find<'apps>(apps: &'apps Apps, id: &str) -> Option<Hit<'apps>> {
    apps.all().find(|hit| hit.entry.id() == id)
}

/// Draws the details of the open app, or Back, which closes them.
pub fn body(page: &mut DiscoverPage, ui: &mut Ui, context: &PageContext<'_>) {
    let Some(open) = page.open.clone() else {
        return;
    };

    // Only a narrow page replaces the list with the details, so only there
    // does a Back have to take the reader out of them.
    if page.narrow && back(ui) {
        page.close();
        return;
    }

    let rest = (Same::of(context.snapshot), context.revision, Look::of(ui));
    let built = page
        .detail
        .get(&[&open.id], rest, || build(ui, context, &open.id));

    let Some(detail) = (*built).as_ref() else {
        page.open = None;
        return;
    };

    ui.add_space(ui.tokens().sizes.card_gap);
    padded_card(ui, |ui| header(ui, context, detail));
    ui.add_space(ui.tokens().sizes.card_gap);

    let catalog = context.service::<Packages>().index().known().cloned();
    padded_card(ui, |ui| sources(ui, detail, catalog.as_deref()));
}

/// The back button, as Kirigami's: a flat button that names the list.
fn back(ui: &mut Ui) -> bool {
    let ctx = ui.ctx().clone();
    let text = i18n::text(&ctx, "discover-back").into_owned();
    let name = i18n::text(&ctx, "discover-back-name").into_owned();
    pill_button_with_icon(ui, &text, ButtonKind::Ghost, icons::GO_PREVIOUS)
        .accessible_name(&name)
        .clicked()
}

fn header(ui: &mut Ui, context: &PageContext<'_>, detail: &Detail) {
    let row = &detail.row;
    let ink = tokens::current(ui.ctx()).text;
    ui.horizontal_top(|ui| {
        let at = picture_slot(ui, false);
        ui.vertical(|ui| {
            section_heading(ui, &row.title, "");
            ui.add(Label::new(rich(
                ui,
                &detail.text,
                Weight::Regular,
                BODY,
                ink,
            )));
            button(ui, context, row);
        });

        row::picture(ui, context, row, at);
    });
}

fn button(ui: &mut Ui, context: &PageContext<'_>, row: &AppRow) {
    ui.horizontal(|ui| {
        if control::press(ui, row::button(context, row)) {
            row::send_toggle(context, row);
        }

        if let Some((note, tone)) = row.note() {
            status_dot(ui, note, tone);
        }
    });
}

/// The packages the app installs, each with its repository when the index
/// knows it, and the website.
fn sources(ui: &mut Ui, detail: &Detail, catalog: Option<&Catalog>) {
    let ctx = ui.ctx().clone();
    let ink = tokens::current(&ctx).text;
    if !detail.packages.is_empty() {
        minor_heading(ui, &i18n::text(&ctx, "discover-details-packages"), "");
        for name in &detail.packages {
            let repository = catalog
                .and_then(|catalog| catalog.entry(name))
                .and_then(|entry| match &entry.origin {
                    Origin::Repository(repository) => Some(repository.as_str()),
                    Origin::Foreign => None,
                });

            let line = repository.map_or_else(
                || name.as_str().to_owned(),
                |repository| {
                    i18n::text_with(
                        &ctx,
                        "discover-package-from",
                        &[
                            ("package", Arg::from(name.as_str())),
                            ("repository", Arg::from(repository)),
                        ],
                    )
                    .into_owned()
                },
            );

            ui.label(rich(ui, &line, Weight::Regular, BODY, ink));
        }
    }

    if let Some(url) = &detail.row.url {
        minor_heading(ui, &i18n::text(&ctx, "discover-details-website"), "");
        let text = i18n::text(&ctx, "row-open-website").into_owned();
        if text_link(ui, &text, &text).clicked() {
            crate::links::open(&ctx, url);
        }
    }
}
