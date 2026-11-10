use std::sync::Arc;

use egui::Ui;
use toolbox_core::features::packages::list::{Catalog, Counts, Filter};
use toolbox_core::features::packages::queue::Refusal;
use toolbox_gui_kit::row_cache::Same;
use toolbox_gui_kit::widgets::{
    ButtonKind, RowHeights, RowList, Tone, banner, counted_chips, dimmed, empty_state,
    loading_indicator, page_toolbar, pill_button,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::PackageName;

use super::row::{self, PackageRow, Row};
use super::{PackagesPage, aur, pending_updates, shows_aur};
use crate::command::PackagesCommand;
use crate::i18n;
use crate::packages::{Packages, Read};
use crate::pages::header::{refresh_header, search};
use crate::pages::rows::refusal_banner;
use crate::pages::{PageContext, plan_running};

struct AurChip {
    /// `None` until the AUR's copy is read.
    count: Option<usize>,
}

pub fn header(page: &mut PackagesPage, ui: &mut Ui, context: &PageContext<'_>) {
    if refresh_header(
        ui,
        "page-packages",
        "page-packages-subtitle",
        "packages-refresh",
    ) {
        context.commands.send(PackagesCommand::Refresh);
    }

    page_toolbar(
        ui,
        page,
        |page, ui| {
            _ = search(
                ui,
                &mut page.search,
                "packages-search",
                "packages-search-hint",
            );
        },
        |_, _| {},
    );
}

pub fn body(page: &mut PackagesPage, ui: &mut Ui, context: &PageContext<'_>) {
    let _ui = tracing::trace_span!("packages ui").entered();
    plan_running(ui, context);
    refusal(ui, context);

    let aur_on = shows_aur(context);
    if !aur_on && page.filter == Filter::Aur {
        page.filter = Filter::Installed;
    }

    // The AUR needs none of pacman's databases, so its chip stays under
    // the notice when they cannot be read.
    let (catalog, unread) = match context.service::<Packages>().index() {
        Read::Known(catalog) => (Arc::clone(catalog), false),
        Read::Loading => {
            ui.vertical_centered(|ui| {
                loading_indicator(ui, &i18n::text(ui.ctx(), "packages-loading"));
            });
            return;
        }
        Read::Unavailable | Read::Failed if !aur_on => {
            unreadable_index(ui, context);
            return;
        }
        Read::Unavailable | Read::Failed => {
            unreadable_index(ui, context);
            ui.add_space(ui.tokens().sizes.header_gap);
            (Arc::new(Catalog::default()), true)
        }
    };

    let updates = pending_updates(context);
    let key = (page.filter, Same::of(&catalog), updates.clone());
    let view = page.view.get(&[&page.search], key, || {
        let _build = tracing::trace_span!("filter packages").entered();
        catalog.view(page.filter, &page.search, &updates)
    });

    let aur_rows = aur_on.then(|| aur::rows_of(page, context)).flatten();
    let aur_chip = aur_on.then(|| AurChip {
        count: aur_rows.as_ref().map(|rows| rows.len()),
    });

    filters(page, ui, view.counts, aur_chip);
    match (aur_on, page.filter) {
        (false, _) => aur::off_hint(ui, context),
        (true, Filter::Aur) => aur::note(ui, context),
        (true, _) => {}
    }

    ui.add_space(ui.tokens().sizes.header_gap);

    if page.filter == Filter::Aur {
        aur::body(
            page,
            ui,
            context,
            &catalog,
            aur_rows.as_ref().map(|rows| rows.as_slice()),
        );
        return;
    }

    if unread {
        return;
    }

    if view.rows.is_empty() {
        nothing(page, ui);
        return;
    }

    rows(page, ui, context, &catalog, &view.rows);
}

fn rows(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    catalog: &Catalog,
    places: &[usize],
) {
    let entries = catalog.entries();
    let shown: Vec<Row<'_>> = places
        .iter()
        .filter_map(|&place| entries.get(place))
        .map(|entry| Row { entry, catalog })
        .collect();

    let list = i18n::text(ui.ctx(), "page-packages");
    if let Some(package) = shown_rows(page, ui, context, &shown, ("packages", &list)) {
        page.open(&package, false, context);
    }
}

/// One list under its id salt and name. Returns the package whose row
/// asked for its detail view, which the caller opens on its own side.
#[must_use = "a dropped package loses the click that asked to open it"]
pub fn shown_rows<R>(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    shown: &[R],
    (salt, list): (&'static str, &str),
) -> Option<PackageName>
where
    R: PackageRow,
{
    let heights = RowHeights::Fixed(row::height(ui));
    let revealed = page.back_to.as_ref().map(|name| name.as_str().to_owned());
    let mut opened = None;

    let holds = RowList::new(salt, list, heights)
        .reveal(revealed.as_deref())
        .show(ui, shown, |ui, item, state| {
            dimmed(ui, context.busy, |ui| {
                if row::show(ui, context, page, item, state) {
                    opened = Some(item.package().clone());
                }
            });
        });

    if holds
        || revealed.is_none()
        || !shown
            .iter()
            .any(|item| Some(item.package().as_str()) == revealed.as_deref())
    {
        page.back_to = None;
    }

    opened
}

fn filters(page: &mut PackagesPage, ui: &mut Ui, counts: Counts, aur: Option<AurChip>) {
    let ctx = ui.ctx().clone();
    let mut chips = vec![
        (
            Filter::Installed,
            "packages-filter-installed",
            Some(counts.installed),
        ),
        (
            Filter::Updates,
            "packages-filter-updates",
            Some(counts.updates),
        ),
        (
            Filter::Orphans,
            "packages-filter-orphans",
            Some(counts.orphans),
        ),
        (Filter::All, "packages-filter-all", Some(counts.all)),
    ];

    if let Some(chip) = aur {
        chips.push((Filter::Aur, "packages-filter-aur", chip.count));
    }

    let texts: Vec<(Filter, String, Option<String>)> = chips
        .iter()
        .map(|(filter, id, count)| {
            (
                *filter,
                i18n::text(&ctx, id).into_owned(),
                count.map(|count| count.to_string()),
            )
        })
        .collect();

    let options: Vec<(Filter, &str, Option<&str>)> = texts
        .iter()
        .map(|(filter, title, count)| (*filter, title.as_str(), count.as_deref()))
        .collect();

    if let Some(picked) = counted_chips(ui, &options, page.filter) {
        page.filter = picked;
    }
}

pub fn nothing(page: &mut PackagesPage, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let search = page.search.trim().to_owned();
    if search.is_empty() {
        let (title, detail) = match page.filter {
            Filter::Updates => ("packages-no-updates", "packages-no-updates-detail"),
            Filter::Orphans => ("packages-no-orphans", "packages-no-orphans-detail"),
            Filter::Installed | Filter::All | Filter::Aur => {
                ("packages-none", "packages-none-detail")
            }
        };

        empty_state(
            ui,
            icons::REPOSITORY,
            &i18n::text(&ctx, title),
            &i18n::text(&ctx, detail),
        );
        return;
    }

    let title = i18n::text_with(&ctx, "packages-nothing-matches", &[("search", &search)]);
    let detail = i18n::text(&ctx, "packages-nothing-matches-detail");
    empty_state(ui, icons::SYSTEM_SEARCH, &title, &detail);
    ui.add_space(ui.tokens().sizes.card_gap);
    ui.vertical_centered(|ui| {
        let clear = i18n::text(&ctx, "packages-clear-search");
        if pill_button(ui, &clear, ButtonKind::Soft).clicked() {
            page.search.clear();
        }
    });
}

fn unreadable_index(ui: &mut Ui, context: &PageContext<'_>) {
    let id = if matches!(context.service::<Packages>().index(), Read::Unavailable) {
        "packages-unavailable"
    } else {
        "packages-failed"
    };

    unreadable(ui, context, id);
}

pub fn unreadable(ui: &mut Ui, context: &PageContext<'_>, id: &str) {
    let ctx = ui.ctx().clone();
    let again = i18n::text(&ctx, "common-read-again");
    if banner(ui, Tone::Warning, &i18n::text(&ctx, id), &[&again]).is_some() {
        context.commands.send(PackagesCommand::Refresh);
    }
}

pub fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(refusal) = &context.refusals.packages else {
        return;
    };

    let package = refusal.package.as_str();
    let protected = |id: &str, name: &str, why: toolbox_plan::Protection| {
        let why = i18n::text(ui.ctx(), &format!("protected-{}", why.as_str()));
        i18n::text_with(ui.ctx(), id, &[("name", name), ("why", why.as_ref())])
    };

    let text = match &refusal.refusal {
        Refusal::Protected(why) => protected("protected-refused", package, *why),
        Refusal::TakesAlong(why, along) => protected("protected-along", along.as_str(), *why),
        other @ (Refusal::NotOffered
        | Refusal::Unread
        | Refusal::RunningKernel
        | Refusal::RunningUnknown
        | Refusal::LastKernel) => i18n::text_with(
            ui.ctx(),
            &format!("packages-refused-{}", other.as_str()),
            &[("package", package)],
        ),
    };

    if refusal_banner(ui, Tone::Error, &text, "packages-dismiss") {
        context.commands.send(PackagesCommand::DismissRefusal);
    }
}
