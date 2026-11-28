mod chips;
mod unread;

use std::collections::BTreeSet;
use std::sync::Arc;

use chips::{AurChip, ChipCounts, filters, idle_filters};
use egui::Ui;
use toolbox_core::features::packages::list::{Catalog, Filter, View};
use toolbox_gui_kit::row_cache::Same;
use toolbox_gui_kit::widgets::{
    ListItem, RowHeights, RowList, Rows, Tone, dimmed, empty_state, page_toolbar,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::PackageName;
use toolbox_services::home::Updates;
pub use unread::unreadable;
use unread::{placeholder, unreadable_index, unreadable_page};

use super::row::{self, PackageRow, Row};
use super::{PackagesPage, aur, shows_aur};
use crate::command::{AppCommand, PackagesCommand};
use crate::i18n;
use crate::packages::{Packages, Read};
use crate::page::PageId;
use crate::pages::PageContext;
use crate::pages::header::{refresh_header_reading, search};
use crate::pages::rows::refusal_banner;
use crate::pages::search_empty::search_empty_state;
use crate::refused::Shown as _;

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
    let packages = context.service::<Packages>();
    let reading = packages.reading();
    let listed = matches!(packages.index(), Read::Known(_))
        || (shows_aur(context) && matches!(packages.aur(), Read::Known(_)));

    if refresh_header_reading(
        ui,
        ("page-packages", "page-packages-subtitle"),
        "packages-refresh",
        reading,
    ) {
        context.commands.send(PackagesCommand::Refresh);
    }

    // The search narrows a list, so it waits for one.
    page_toolbar(
        ui,
        page,
        |page, ui| {
            ui.enabled_ui(listed, |ui| {
                _ = search(
                    ui,
                    &mut page.search,
                    "packages-search",
                    "packages-search-hint",
                );
            });
        },
        |_, _| {},
    );
}

/// The list's top, which spans the page above both columns: the chips, the
/// AUR's lines and the notice of a repository index that cannot be read.
pub fn top(page: &mut PackagesPage, ui: &mut Ui, context: &PageContext<'_>) {
    let aur_on = shows_aur(context);
    if !aur_on && page.filter == Filter::Aur {
        page.filter = Filter::Installed;
    }

    match context.service::<Packages>().index() {
        Read::Known(catalog) => {
            let catalog = Arc::clone(catalog);
            known_top(page, ui, context, &catalog, false);
        }
        Read::Loading => {
            ui.enabled_ui(false, |ui| idle_filters(page, ui, aur_on));
        }
        Read::Unavailable | Read::Failed if !aur_on => idle_filters(page, ui, aur_on),
        Read::Unavailable | Read::Failed => {
            // Only the AUR's chip has rows without pacman's databases.
            page.filter = Filter::Aur;
            unreadable_index(ui, context);
            ui.add_space(ui.tokens().sizes.header_gap);
            let catalog = Arc::new(Catalog::default());
            known_top(page, ui, context, &catalog, true);
        }
    }
}

/// The list's column under the top: the rows, or the state that stands in
/// for them while the index is read or cannot be.
pub fn column(page: &mut PackagesPage, ui: &mut Ui, context: &PageContext<'_>) {
    let aur_on = shows_aur(context);
    match context.service::<Packages>().index() {
        Read::Known(catalog) => {
            let catalog = Arc::clone(catalog);
            known_column(page, ui, context, &catalog, aur_on);
        }
        Read::Loading => placeholder(ui, "packages-loading", page.open.is_some()),
        Read::Unavailable | Read::Failed if !aur_on => unreadable_page(ui, context),
        Read::Unavailable | Read::Failed => {
            let catalog = Arc::new(Catalog::default());
            known_column(page, ui, context, &catalog, aur_on);
        }
    }
}

/// The chips and the AUR's lines of a read index. `unread` leaves the
/// repository chips off, as the AUR alone lists then.
fn known_top(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    catalog: &Arc<Catalog>,
    unread: bool,
) {
    let aur_on = shows_aur(context);
    let updates = Updates::of(context.home.status_read());
    let view = view_of(page, catalog, &updates, context);
    let aur_rows = aur_on.then(|| aur::rows_of(page, context)).flatten();
    let counts = ChipCounts {
        repository: (!unread).then_some(view.counts),
        updates_known: updates.is_known(),
        aur: aur_on.then(|| AurChip {
            count: aur_rows.as_ref().map(|rows| rows.len()),
        }),
    };

    filters(page, ui, counts);
    match (aur_on, page.filter) {
        (false, _) => aur::off_hint(ui, context),
        (true, Filter::Aur) => aur::note(ui, context),
        (true, _) => {}
    }
}

/// The rows of a read index, or the AUR's list or the updates' state the
/// chip picked.
fn known_column(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    catalog: &Arc<Catalog>,
    aur_on: bool,
) {
    if page.filter == Filter::Aur {
        let aur_rows = aur_on.then(|| aur::rows_of(page, context)).flatten();
        let places = aur_rows.as_ref().map(|rows| rows.as_slice());
        aur::body(page, ui, context, catalog, places);
        return;
    }

    let updates = Updates::of(context.home.status_read());
    if page.filter == Filter::Updates && !updates_read(ui, context, &updates, page.open.is_some()) {
        return;
    }

    let view = view_of(page, catalog, &updates, context);
    if view.rows.is_empty() {
        nothing(page, ui);
        return;
    }

    rows(page, ui, context, catalog, &view.rows);
}

/// The view of the repositories' catalog under the chip and the search.
fn view_of(
    page: &mut PackagesPage,
    catalog: &Arc<Catalog>,
    updates: &Updates,
    context: &PageContext<'_>,
) -> Arc<View> {
    let key = (page.filter, Same::of(catalog), context.home.status_revision);
    page.view.get(&[&page.search], key, || {
        let _build = tracing::trace_span!("filter packages").entered();
        let pending = match updates {
            Updates::Known(pending) => pending.clone(),
            Updates::Unread | Updates::Failed => BTreeSet::new(),
        };

        catalog.view(page.filter, &page.search, &pending)
    })
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
    if let Some(package) = shown_rows(page, ui, context, &shown, ("packages", &list, false)) {
        page.open(&package, false, context);
    }
}

/// One list under its id salt and name. Returns the package whose row
/// asked for its detail view, which the caller opens on its own side. The
/// row of the open package is selected when `aur` is the side it opened on.
#[must_use = "a dropped package loses the click that asked to open it"]
pub(super) fn shown_rows<'rows, S>(
    page: &mut PackagesPage,
    ui: &mut Ui,
    context: &PageContext<'_>,
    shown: &'rows S,
    (salt, list, aur): (&'static str, &str, bool),
) -> Option<PackageName>
where
    S: Rows,
    S::Item<'rows>: PackageRow,
{
    let heights = RowHeights::Fixed(row::height(ui, page.open.is_some()));
    let revealed = page.back_to.as_ref().map(|name| name.as_str().to_owned());
    let selected = page.selection(aur);
    let mut opened = None;

    let holds = RowList::new(salt, list, heights)
        .reveal(revealed.as_deref())
        .selected(selected.as_deref())
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
fn updates_read(ui: &mut Ui, context: &PageContext<'_>, updates: &Updates, compact: bool) -> bool {
    match updates {
        Updates::Known(_) => true,
        Updates::Unread => {
            placeholder(ui, "packages-updates-loading", compact);
            false
        }
        Updates::Failed => {
            unreadable(ui, context, "packages-updates-failed");
            false
        }
    }
}

pub fn nothing(page: &mut PackagesPage, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let search = page.search.trim().to_owned();
    if search.is_empty() {
        let (title, detail) = match page.filter {
            Filter::Updates => ("packages-no-updates", "packages-no-updates-detail"),
            Filter::Orphans => ("packages-no-orphans", "packages-no-orphans-detail"),
            Filter::Optional => ("packages-no-optional", "packages-no-optional-detail"),
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

    let detail = i18n::text(&ctx, "packages-nothing-matches-detail");
    if search_empty_state(ui, &search, &detail) {
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
