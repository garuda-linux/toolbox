mod aur;
mod back;
mod below;
mod facts;
mod relations;
#[cfg(test)]
mod tests;
mod title;

use std::sync::Arc;

use back::Back;
pub use below::{FileRows, HistoryRows};
use egui::{Align, Layout, Ui, vec2};
pub use facts::FactRows;
pub use relations::TargetRows;
use title::{title_picture, title_picture_width};
use toolbox_core::features::packages::PackageDetails;
use toolbox_core::features::packages::list::Catalog;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::row_cache::Same;
use toolbox_gui_kit::widgets::{
    ButtonKind, PageHeader, empty_state, icon_button_width, loading_indicator, padded_card,
    pill_width, refresh_button_busy, rich, status_dot,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
use toolbox_plan::PackageName;

use super::PackagesPage;
use super::row::{self, Act, PackageRow, Row};
use crate::command::PackagesCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::packages::{Details, Packages, Read};
use crate::pages::PageContext;
use crate::pages::rows::Look;

/// The details' heading, with the back button when it leads somewhere.
fn header_of<R>(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    package: &PackageName,
    shown: Option<R>,
) where
    R: PackageRow,
{
    let description = shown.as_ref().map_or("", PackageRow::description);
    let button = shown.map(|shown| row::button_text(ui, context, &shown));

    // The row's item spacing follows the button before the card gap does.
    let button_width = button.as_ref().map_or(0.0, |(text, _)| {
        pill_width(ui, text) + ui.spacing().item_spacing.x + ui.tokens().sizes.card_gap
    });

    let trailing = button_width + icon_button_width();
    let back = Back::new(page, ui, trailing);
    let back_shown = page.narrow || !page.trail.is_empty();

    let mut refresh = false;
    let mut pressed = false;
    ui.horizontal_top(|line| {
        line.vertical(|column| {
            let gap = column.tokens().sizes.text_to_control;
            let before = if back_shown { back.width + gap } else { 0.0 };
            let lead = before + title_picture_width(column);
            PageHeader::new(package.as_str(), description)
                .leading(lead, |leading| {
                    if back_shown {
                        let row = leading.tokens().sizes.toolbar_height;
                        leading.allocate_ui_with_layout(
                            vec2(back.width, row),
                            Layout::left_to_right(Align::Center),
                            |back_ui| pressed = back.show(back_ui),
                        );

                        leading.add_space(gap);
                    }

                    title_picture(leading, context, package);
                })
                .trailing(trailing, |actions| {
                    if let (Some(shown), Some(button)) = (shown, &button) {
                        let (text, act) = button;
                        if row::action(actions, context, &shown, text, strength(*act)) {
                            shown.act(page, context, *act);
                        }

                        actions.add_space(actions.tokens().sizes.card_gap);
                    }

                    let name = i18n::text(actions.ctx(), "packages-refresh");
                    let busy = context
                        .service::<Packages>()
                        .reading()
                        .then(|| i18n::text(actions.ctx(), "common-reading").into_owned());

                    refresh = refresh_button_busy(actions, &name, busy.as_deref())
                        .value
                        .is_some();
                })
                .show(column);

            if let Some(shown) = shown {
                facts_line(column, context, &shown);
            }
        });
    });

    if refresh {
        context.commands.send(PackagesCommand::Refresh);
    }

    if pressed {
        page.back(context);
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

    if page.open_aur {
        aur::header(page, ui, context, &package);
        aur::body(page, ui, context, &package);
        return;
    }

    let catalog = catalog(context);
    let entry = catalog.as_ref().and_then(|catalog| catalog.entry(&package));
    let shown = catalog
        .as_deref()
        .zip(entry)
        .map(|(catalog, entry)| Row { entry, catalog });

    header_of(page, ui, context, &package, shown);

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
            columns(ui, page, context, catalog.as_ref(), (details, info));
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
    catalog: Option<&Arc<Catalog>>,
    (details, info): (&Arc<Details>, &PackageDetails),
) {
    let facts = page.facts.get(&[], (Same::of(details), Look::of(ui)), || {
        let _build = tracing::trace_span!("build facts").entered();
        facts::facts(ui.ctx(), info)
    });

    let targets = page
        .targets
        .get(&[], (Same::of(details), catalog.map(Same::of)), || {
            relations::resolve(catalog.map(Arc::as_ref), info)
        });

    let unfolded = &mut page.unfolded;
    let opened = split(
        ui,
        |ui| facts::grid(ui, &facts),
        |ui| relations::show(ui, unfolded, &targets, info),
    );

    if let Some(package) = opened {
        page.follow(&package, false, context);
    }
}

/// The facts and the relations, each in its own card, one under the other.
/// Returns the package whose name was clicked.
fn split(
    ui: &mut Ui,
    left: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui) -> Option<PackageName>,
) -> Option<PackageName> {
    padded_card(ui, left);
    ui.add_space(ui.tokens().sizes.card_gap);
    padded_card(ui, right).inner
}

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
    let title = i18n::text_with(
        &ctx,
        "packages-gone",
        &[("package", Arg::from(package.as_str()))],
    );

    empty_state(
        ui,
        icons::REPOSITORY,
        &title,
        &i18n::text(&ctx, "packages-gone-detail"),
    );

    ui.add_space(ui.tokens().sizes.card_gap);
}
