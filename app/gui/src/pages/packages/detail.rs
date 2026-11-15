mod aur;
mod below;
mod facts;
mod relations;
#[cfg(test)]
mod tests;

use std::sync::Arc;

pub use below::{FileRows, HistoryRows};
use egui::{Color32, Key, Rect, Sense, Ui, Vec2, pos2, vec2};
pub use facts::{FactRows, day};
use toolbox_core::features::packages::PackageDetails;
use toolbox_core::features::packages::list::Catalog;
use toolbox_gui_kit::fonts::{self, SMALL, TITLE, Weight};
use toolbox_gui_kit::row_cache::Same;
use toolbox_gui_kit::widgets::{
    ButtonKind, empty_state, icon_button_width, loading_indicator, named_pill_button, page_header,
    pill_width, refresh_button, rich, status_dot,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
use toolbox_plan::PackageName;

use super::PackagesPage;
use super::row::{self, Act, PackageRow, Row};
use crate::artwork::Artwork;
use crate::command::PackagesCommand;
use crate::i18n;
use crate::packages::{Details, Packages, Read};
use crate::pages::rows::{Initial, Look, package_picture};
use crate::pages::{PageContext, plan_running};
use crate::share::view::padded_card;

/// The content width under which the two columns stack: about a window
/// of 760 points beside the open sidebar.
pub const STACK_BELOW: f32 = 560.0;

pub fn header(page: &mut PackagesPage, ui: &mut Ui, context: &PageContext<'_>) {
    let Some(package) = page.open.clone() else {
        return;
    };

    if page.open_aur {
        aur::header(page, ui, context, &package);
        return;
    }

    let catalog = catalog(context);
    let entry = catalog.as_ref().and_then(|catalog| catalog.entry(&package));
    let shown = catalog
        .as_deref()
        .zip(entry)
        .map(|(catalog, entry)| Row { entry, catalog });

    header_of(page, ui, context, &package, shown);
}

/// The back button, then the picture, name and description with the
/// row's button and the facts line once the package is known.
fn header_of<R>(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    package: &PackageName,
    shown: Option<R>,
) where
    R: PackageRow,
{
    back(page, ui, context);
    ui.add_space(ui.tokens().sizes.card_gap);

    let description = shown.as_ref().map_or("", PackageRow::description);
    let button = shown.map(|shown| row::button_text(ui, context, &shown));
    let button_width = button.as_ref().map_or(0.0, |(text, _)| {
        pill_width(ui, text) + ui.tokens().sizes.card_gap
    });

    let mut refresh = false;
    ui.horizontal_top(|ui| {
        title_picture(ui, context, package);
        ui.vertical(|ui| {
            let width = button_width + icon_button_width(ui);
            page_header(ui, package.as_str(), description, width, |ui| {
                if let (Some(shown), Some(button)) = (shown, &button) {
                    let (text, act) = button;
                    if row::action(ui, context, &shown, text, strength(*act)) {
                        shown.act(page, context, *act);
                    }

                    ui.add_space(ui.tokens().sizes.card_gap);
                }

                refresh = refresh_button(ui, &i18n::text(ui.ctx(), "packages-refresh"));
            });

            if let Some(shown) = shown {
                facts_line(ui, context, &shown);
            }
        });
    });

    if refresh {
        context.commands.send(PackagesCommand::Refresh);
    }
}

/// Installing is what a package's page leads to, so it is the page's one
/// Primary. Remove opens a question and Take back is a peer of the queue,
/// both stay Soft.
const fn strength(act: Act) -> ButtonKind {
    match act {
        Act::Install => ButtonKind::Primary,
        Act::Remove | Act::TakeBack(_) => ButtonKind::Soft,
    }
}

/// It is as tall as the title's line and starts on the top of its
/// capitals, so it stands as large as the name and level with its letters.
fn title_picture(ui: &mut Ui, context: &PageContext<'_>, package: &PackageName) {
    let font = fonts::font(ui.style(), Weight::Bold, TITLE);
    let capital = ui
        .painter()
        .layout_no_wrap("H".to_owned(), font, Color32::PLACEHOLDER);

    let size = capital.rect.height();
    let top = capital.mesh_bounds.top();
    let (room, _) = ui.allocate_exact_size(vec2(size, top + size), Sense::hover());
    let slot = Rect::from_min_size(pos2(room.left(), room.top() + top), Vec2::splat(size));
    package_picture(
        ui,
        context.service::<Artwork>().pictures(),
        Some(package),
        Initial::named(package),
        slot,
    );

    ui.add_space(ui.tokens().sizes.picture_gap);
}

/// The removal question answers Escape itself.
fn back(page: &mut PackagesPage, ui: &mut Ui, context: &PageContext<'_>) {
    let escape_free = page.removing.is_none() && page.installing.is_none();
    let ctx = ui.ctx().clone();
    let (text, name) = page.trail.last().map_or_else(
        || {
            (
                i18n::text(&ctx, "packages-back"),
                i18n::text(&ctx, "packages-back-name"),
            )
        },
        |(previous, _)| {
            let to = [("package", previous.as_str())];
            (
                i18n::text_with(&ctx, "packages-back-to", &to),
                i18n::text_with(&ctx, "packages-back-to-name", &to),
            )
        },
    );

    let pressed = named_pill_button(ui, &text, ButtonKind::Soft, &name).clicked();
    let escape =
        escape_free && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, Key::Escape));

    if pressed || escape {
        page.back(context);
    }
}

fn facts_line<R>(ui: &mut Ui, context: &PageContext<'_>, shown: &R)
where
    R: PackageRow,
{
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let text = shown.header_line(&ctx);

    ui.horizontal(|ui| {
        ui.label(rich(ui, &text, Weight::Regular, SMALL, tokens.secondary));
        if let Some((id, tone)) = row::state_word(context, shown) {
            status_dot(ui, &i18n::text(&ctx, id), tone);
        }
    });
}

pub fn body(page: &mut PackagesPage, ui: &mut Ui, context: &PageContext<'_>) {
    let Some(package) = page.open.clone() else {
        return;
    };

    plan_running(ui, context);
    super::list::refusal(ui, context);

    if page.open_aur {
        aur::body(page, ui, context, &package);
        return;
    }

    let Some(details) = context
        .service::<Packages>()
        .details()
        .filter(|details| details.package == package)
    else {
        loading(ui);
        return;
    };

    match &details.info {
        Read::Loading => loading(ui),
        Read::Unavailable | Read::Failed => {
            super::list::unreadable(ui, context, "packages-details-failed");
        }
        Read::Known(None) => gone(ui, &package),
        Read::Known(Some(info)) => {
            let catalog = catalog(context);
            columns(ui, page, context, catalog.as_deref(), (details, info));
            if info.installed.is_some() {
                group_gap(ui);
                below::files(page, ui, context, (details, info));
            }

            group_gap(ui);
            below::history(page, ui, context, details);
        }
    }
}

fn columns(
    ui: &mut Ui,
    page: &mut PackagesPage,
    context: &PageContext<'_>,
    catalog: Option<&Catalog>,
    (details, info): (&Arc<Details>, &PackageDetails),
) {
    let facts = page.facts.get(&[], (Same::of(details), Look::of(ui)), || {
        let _build = tracing::trace_span!("build facts").entered();
        facts::facts(ui.ctx(), info)
    });

    let unfolded = &mut page.unfolded;
    let opened = split(
        ui,
        |ui| facts::grid(ui, &facts),
        |ui| relations::show(ui, unfolded, catalog, info),
    );

    if let Some(package) = opened {
        page.follow(&package, false, context);
    }
}

/// The facts and the relations, each in its own card, side by side when
/// there is room. Returns the package whose name was clicked.
fn split(
    ui: &mut Ui,
    left: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui) -> Option<PackageName>,
) -> Option<PackageName> {
    if ui.available_width() < STACK_BELOW {
        padded_card(ui, left);
        ui.add_space(ui.tokens().sizes.card_gap);
        return padded_card(ui, right).inner;
    }

    let spacing = ui.spacing().item_spacing.x;
    let mut opened = None;
    ui.scope(|ui| {
        // The gap between the columns only, their own rows keep theirs.
        ui.spacing_mut().item_spacing.x = ui.tokens().sizes.card_gap;
        ui.columns(2, |columns| {
            if let [left_ui, right_ui] = columns {
                left_ui.spacing_mut().item_spacing.x = spacing;
                right_ui.spacing_mut().item_spacing.x = spacing;
                padded_card(left_ui, left);
                opened = padded_card(right_ui, right).inner;
            }
        });
    });

    opened
}

/// The room between two groups of the page.
fn group_gap(ui: &mut Ui) {
    ui.add_space(ui.tokens().sizes.section_gap);
}

fn catalog(context: &PageContext<'_>) -> Option<Arc<Catalog>> {
    context
        .service::<Packages>()
        .index()
        .known()
        .map(Arc::clone)
}

fn loading(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        loading_indicator(ui, &i18n::text(ui.ctx(), "packages-details-loading"));
    });
}

/// A database no longer has the package, such as after a removal.
fn gone(ui: &mut Ui, package: &PackageName) {
    let ctx = ui.ctx().clone();
    let title = i18n::text_with(&ctx, "packages-gone", &[("package", package.as_str())]);
    empty_state(
        ui,
        icons::REPOSITORY,
        &title,
        &i18n::text(&ctx, "packages-gone-detail"),
    );
    ui.add_space(ui.tokens().sizes.card_gap);
}
