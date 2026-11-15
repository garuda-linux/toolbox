mod unread;

use std::collections::BTreeSet;
use std::sync::Arc;

use egui::{Ui, vec2};
use toolbox_core::features::packages::list::{Catalog, Counts, Filter};
use toolbox_gui_kit::row_cache::Same;
use toolbox_gui_kit::widgets::{
    ListItem, RowHeights, RowList, Rows, Tone, chip, dimmed, empty_state, empty_state_action,
    page_toolbar,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::PackageName;
pub use unread::unreadable;
use unread::{placeholder, unreadable_index, unreadable_page, waiting};

use super::row::{self, PackageRow, Row};
use super::{PackagesPage, Updates, aur, shows_aur};
use crate::command::{AppCommand, PackagesCommand};
use crate::i18n;
use crate::packages::{Packages, Read};
use crate::page::PageId;
use crate::pages::header::{refresh_header, search};
use crate::pages::rows::refusal_banner;
use crate::pages::{PageContext, plan_running};
use crate::refused::Shown as _;

struct AurChip {
    /// `None` until the AUR's copy is read.
    count: Option<usize>,
}

/// What the chips count. A count that is not known shows none.
struct ChipCounts {
    /// `None` without pacman's databases.
    repository: Option<Counts>,
    updates_known: bool,
    /// `None` while the AUR is off.
    aur: Option<AurChip>,
}

/// The rows of `places` in `entries`, each built only when the list draws
/// it: the AUR alone has about a hundred thousand.
pub struct Picked<'a, E, F> {
    pub places: &'a [usize],
    pub entries: &'a [E],
    pub row: F,
}

impl<'a, E, I, F> Rows for Picked<'a, E, F>
where
    I: ListItem,
    F: Fn(&'a E) -> I,
{
    type Item<'b>
        = I
    where
        Self: 'b;

    fn len(&self) -> usize {
        self.places.len()
    }

    fn get(&self, index: usize) -> Option<I> {
        let place = *self.places.get(index)?;
        self.entries.get(place).map(&self.row)
    }
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

    let Some((catalog, unread)) = read_catalog(page, ui, context, aur_on) else {
        return;
    };

    // Only the AUR's chip has rows without pacman's databases.
    if unread {
        page.filter = Filter::Aur;
    }

    let updates = Updates::of(context);
    let status = context.home.status_revision;
    let key = (page.filter, Same::of(&catalog), status);
    let view = page.view.get(&[&page.search], key, || {
        let _build = tracing::trace_span!("filter packages").entered();
        let pending = match &updates {
            Updates::Known(pending) => pending.clone(),
            Updates::Unread | Updates::Failed => BTreeSet::new(),
        };

        catalog.view(page.filter, &page.search, &pending)
    });

    let aur_rows = aur_on.then(|| aur::rows_of(page, context)).flatten();
    let aur_chip = aur_on.then(|| AurChip {
        count: aur_rows.as_ref().map(|rows| rows.len()),
    });

    let counts = ChipCounts {
        repository: (!unread).then_some(view.counts),
        updates_known: updates.is_known(),
        aur: aur_chip,
    };

    filters(page, ui, counts);
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

    if page.filter == Filter::Updates && !updates_read(ui, context, &updates) {
        return;
    }

    if view.rows.is_empty() {
        nothing(page, ui);
        return;
    }

    rows(page, ui, context, &catalog, &view.rows);
}

/// The repositories' catalog and whether it could not be read, `None` when
/// the page shows nothing else.
fn read_catalog(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    aur_on: bool,
) -> Option<(Arc<Catalog>, bool)> {
    // The AUR needs none of pacman's databases, so its chip stays under
    // the notice when they cannot be read.
    match context.service::<Packages>().index() {
        Read::Known(catalog) => Some((Arc::clone(catalog), false)),
        Read::Loading => {
            waiting(page, ui, aur_on);
            None
        }
        Read::Unavailable | Read::Failed if !aur_on => {
            unreadable_page(ui, context);
            None
        }
        Read::Unavailable | Read::Failed => {
            unreadable_index(ui, context);
            ui.add_space(ui.tokens().sizes.header_gap);
            Some((Arc::new(Catalog::default()), true))
        }
    }
}

fn rows(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    catalog: &Catalog,
    places: &[usize],
) {
    let shown = Picked {
        places,
        entries: catalog.entries(),
        row: |entry| Row { entry, catalog },
    };

    let list = i18n::text(ui.ctx(), "page-packages");
    if let Some(package) = shown_rows(page, ui, context, &shown, ("packages", &list)) {
        page.open(&package, false, context);
    }
}

/// One list under its id salt and name. Returns the package whose row
/// asked for its detail view, which the caller opens on its own side.
#[must_use = "a dropped package loses the click that asked to open it"]
pub fn shown_rows<'rows, S>(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    shown: &'rows S,
    (salt, list): (&'static str, &str),
) -> Option<PackageName>
where
    S: Rows,
    S::Item<'rows>: PackageRow,
{
    let heights = RowHeights::Fixed(row::height(ui));
    let revealed = page.back_to.as_ref().map(|name| name.as_str().to_owned());
    let mut opened = None;

    let holds = RowList::new(salt, list, heights)
        .reveal(revealed.as_deref())
        .show(ui, shown, |ui, item, state| {
            dimmed(ui, context.busy, |ui| {
                if row::show(ui, context, page, &item, state) {
                    opened = Some(item.package().clone());
                }
            });
        });

    if holds
        || revealed.is_none()
        || !(0..shown.len()).any(|index| {
            shown
                .get(index)
                .is_some_and(|item| Some(item.package().as_str()) == revealed.as_deref())
        })
    {
        page.back_to = None;
    }

    opened
}

/// Draws the placeholder rows or the failure banner of the Updates chip.
/// Returns whether the updates are known, so the rows may show.
fn updates_read(ui: &mut Ui, context: &PageContext<'_>, updates: &Updates) -> bool {
    match updates {
        Updates::Known(_) => true,
        Updates::Unread => {
            placeholder(ui, "packages-updates-loading");
            false
        }
        Updates::Failed => {
            unreadable(ui, context, "packages-updates-failed");
            false
        }
    }
}

/// The repository chips go off without pacman's databases, as their
/// counts would only be zeros. The AUR's chip needs none of them.
fn filters(page: &mut PackagesPage, ui: &mut Ui, counts: ChipCounts) {
    let ctx = ui.ctx().clone();
    let repository = counts.repository;
    let count = |pick: fn(Counts) -> usize| repository.map(pick);
    let mut chips = vec![
        (
            Filter::Installed,
            "packages-filter-installed",
            count(|known| known.installed),
        ),
        (
            Filter::Updates,
            "packages-filter-updates",
            count(|known| known.updates).filter(|_| counts.updates_known),
        ),
        (
            Filter::Orphans,
            "packages-filter-orphans",
            count(|known| known.orphans),
        ),
        (Filter::All, "packages-filter-all", count(|known| known.all)),
    ];

    if let Some(aur_chip) = counts.aur {
        chips.push((Filter::Aur, "packages-filter-aur", aur_chip.count));
    }

    let gap = ui.tokens().sizes.control_gap;
    let mut picked = None;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(gap, gap);
        for (filter, id, shown) in chips {
            let title = i18n::text(&ctx, id);
            let number = shown.map(|number| number.to_string());
            let enabled = repository.is_some() || filter == Filter::Aur;
            let selected = filter == page.filter;
            let response = ui.add_enabled(enabled, |ui: &mut Ui| {
                chip(ui, &title, number.as_deref(), selected)
            });

            if response.clicked() {
                picked = Some(filter);
            }
        }
    });

    if let Some(filter) = picked {
        page.filter = filter;
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
    let clear = i18n::text(&ctx, "packages-clear-search");
    if empty_state_action(ui, icons::SYSTEM_SEARCH, &title, &detail, &clear) {
        page.search.clear();
    }
}

pub fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(refusal) = context.service::<Packages>().refusal() else {
        return;
    };

    let text = refusal.text(ui.ctx());
    if refusal_banner(ui, Tone::Error, &text, "packages-dismiss") {
        context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Packages));
    }
}
