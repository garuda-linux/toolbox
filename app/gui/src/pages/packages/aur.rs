//! The AUR chip: the AUR's copy searched in the order people look for a
//! package, one row per package, only the visible ones laid out, and the
//! question before a build. Repository and AUR packages never share a
//! list.

pub mod install;
pub mod row;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::{Label, Ui};
use toolbox_core::features::packages::list::{Catalog, Filter};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::row_cache::Same;
use toolbox_gui_kit::widgets::{Tone, banner, loading_indicator, rich, text_link};
use toolbox_gui_kit::{UiExt as _, tokens};

use self::row::AurRow;
use super::PackagesPage;
use super::detail::day;
use crate::command::{PackagesCommand, UiCommand};
use crate::i18n;
use crate::packages::{Packages, Read};
use crate::page::PageId;
use crate::pages::PageContext;

/// The places of the AUR rows under the search, once the copy is read.
/// The chip asks for the copy the first time it shows, never before:
/// reading it may download the dump.
pub fn rows_of(page: &mut PackagesPage, context: &PageContext<'_>) -> Option<Arc<Vec<usize>>> {
    if page.filter == Filter::Aur && !page.aur_asked {
        page.aur_asked = true;
        context.commands.send(PackagesCommand::OpenAur);
    }

    let catalog = context.service::<Packages>().aur().known()?;
    let search = page.search.clone();
    Some(page.aur_view.get(&[&search], Same::of(catalog), || {
        let _build = tracing::trace_span!("filter aur packages").entered();
        catalog.view(&search)
    }))
}

/// The AUR is off: one line under the chips with the way to turn it on.
pub fn off_hint(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    under_chips(ui, |ui| {
        let hint = i18n::text(&ctx, "packages-aur-off");
        ui.add(Label::new(rich(ui, &hint, Weight::Regular, SMALL, tokens.secondary)).wrap());
        let link = i18n::text(&ctx, "packages-aur-off-settings");
        if text_link(ui, &link, &link).clicked() {
            context.commands.send(UiCommand::Navigate(PageId::Settings));
        }
    });
}

/// The line under the chips, one height in either view, so its text and
/// the list under it stay put when the setting changes.
fn under_chips(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    ui.add_space(ui.tokens().sizes.tight_gap);
    super::text_line(ui, add);
}

/// The AUR chip's list: the copy's state, the rows or why there are none.
pub fn body(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    repo: &Catalog,
    places: Option<&[usize]>,
) {
    let ctx = ui.ctx().clone();
    let catalog = match context.service::<Packages>().aur() {
        Read::Known(catalog) => Arc::clone(catalog),
        Read::Loading => {
            ui.vertical_centered(|ui| {
                loading_indicator(ui, &i18n::text(&ctx, "packages-aur-loading"));
            });
            return;
        }
        // No Read again: the copy downloads at most once a day, a click
        // would only read the same answer.
        Read::Unavailable => {
            banner(
                ui,
                Tone::Warning,
                &i18n::text(&ctx, "packages-aur-waiting"),
                &[],
            );
            return;
        }
        Read::Failed => {
            banner(
                ui,
                Tone::Warning,
                &i18n::text(&ctx, "packages-aur-failed"),
                &[],
            );
            return;
        }
    };

    let places = places.unwrap_or_default();
    if places.is_empty() {
        super::list::nothing(page, ui);
        return;
    }

    let entries = catalog.entries();
    let shown: Vec<AurRow<'_>> = places
        .iter()
        .filter_map(|&place| entries.get(place))
        .map(|entry| AurRow { entry, repo })
        .collect();

    let list = i18n::text(&ctx, "packages-filter-aur");
    let rows = ("packages-aur", list.as_ref());
    if let Some(package) = super::list::shown_rows(page, ui, context, &shown, rows) {
        page.open(&package, true, context);
    }
}

/// Where the list comes from and what its packages are, one line under
/// the chips as [`off_hint`] is, once the copy is read.
pub fn note(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(catalog) = context.service::<Packages>().aur().known() else {
        return;
    };

    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let text = i18n::text_with(
        &ctx,
        "packages-aur-note",
        &[("date", &day(catalog.downloaded()))],
    );

    let stale = catalog
        .stale()
        .then(|| i18n::text(&ctx, "packages-aur-stale"));

    under_chips(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.add(Label::new(rich(ui, &text, Weight::Regular, SMALL, tokens.secondary)).wrap());

            if let Some(stale) = stale {
                ui.add(Label::new(rich(ui, &stale, Weight::Regular, SMALL, tokens.warning)).wrap());
            }
        });
    });
}
