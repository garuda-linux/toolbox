mod aur;
mod below;
mod facts;
mod relations;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::{Color32, Key, Rect, Sense, Stroke, Ui, Vec2, pos2, vec2};
pub use facts::day;
use toolbox_core::features::packages::PackageDetails;
use toolbox_core::features::packages::list::Catalog;
use toolbox_gui_kit::fonts::{self, SMALL, TITLE, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, empty_state, icon_button_width, loading_indicator, named_pill_button, page_header,
    pill_width, refresh_button, rich, status_dot,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
use toolbox_plan::PackageName;

use super::PackagesPage;
use super::row::{self, PackageRow, Row};
use crate::command::PackagesCommand;
use crate::i18n;
use crate::packages::{Packages, Read};
use crate::pages::rows::{Initial, package_picture};
use crate::pages::{PageContext, plan_running};

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
                    row::action(ui, page, context, &shown, button);
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
        context.pictures,
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
            columns(ui, page, context, catalog.as_deref(), info);
            rule(ui);
            below::files(page, ui, context, info);
            ui.add_space(ui.tokens().sizes.section_gap);
            below::history(page, ui, context, &details.history);
        }
    }
}

fn columns(
    ui: &mut Ui,
    page: &mut PackagesPage,
    context: &PageContext<'_>,
    catalog: Option<&Catalog>,
    info: &PackageDetails,
) {
    let unfolded = &mut page.unfolded;
    let opened = split(
        ui,
        |ui| facts::show(ui, info),
        |ui| relations::show(ui, unfolded, catalog, info),
    );

    if let Some(package) = opened {
        page.follow(&package, false, context);
    }
}

/// Returns the package whose name was clicked.
fn split(
    ui: &mut Ui,
    left: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui) -> Option<PackageName>,
) -> Option<PackageName> {
    if ui.available_width() < STACK_BELOW {
        left(ui);
        rule(ui);
        return right(ui);
    }

    let top = ui.cursor().top();
    let middle = ui.cursor().left() + ui.available_width() / 2.0;
    let spacing = ui.spacing().item_spacing.x;
    let mut opened = None;
    ui.scope(|ui| {
        // The gap between the columns only, their own rows keep theirs.
        ui.spacing_mut().item_spacing.x = ui.tokens().sizes.section_gap;
        ui.columns(2, |columns| {
            if let [left_ui, right_ui] = columns {
                left_ui.spacing_mut().item_spacing.x = spacing;
                right_ui.spacing_mut().item_spacing.x = spacing;
                left(left_ui);
                opened = right(right_ui);
            }
        });
    });

    let bottom = ui.cursor().top();
    let outline = ui.tokens().outline;
    ui.painter()
        .vline(middle, top..=bottom, Stroke::new(1.0, outline));
    opened
}

fn rule(ui: &mut Ui) {
    let gap = ui.tokens().sizes.card_gap * 2.0;
    ui.add_space(gap);
    let width = ui.available_width();
    let (line, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    let outline = ui.tokens().outline;
    ui.painter()
        .hline(line.x_range(), line.center().y, Stroke::new(1.0, outline));
    ui.add_space(gap);
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
