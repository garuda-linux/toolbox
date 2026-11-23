mod app_row;
mod chips;
mod details;
mod notices;
mod row;
mod sticky;
#[cfg(test)]
mod tests;

use app_row::{AppGroup, AppRow};
use chips::Chips;
use egui::{Key, Modifiers, Ui};
use notices::notices;
use toolbox_core::Snapshot;
use toolbox_core::features::discover::{AppOrder, view};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::columns::{self, Columns};
use toolbox_gui_kit::widgets::{
    BodyShape, RowHeights, RowList, counted_chips, dimmed, empty_state, empty_state_action,
    page_toolbar, section_heading,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::header::{search, title_header};
use super::rows::{Look, sort_select};
use super::search_empty::search_empty_state;
use super::{PageContext, PageView, plan_running};
use crate::i18n;
use crate::i18n::Arg;

#[derive(Debug, Default)]
pub struct DiscoverPage {
    search: String,
    category: Option<String>,
    order: AppOrder,
    rows: RowCache<RowsKey, Vec<AppGroup>>,
    chips: RowCache<ChipsKey, Chips>,
    detail: RowCache<DetailKey, Option<details::Detail>>,
    open: Option<Open>,
    /// Whether the page is too narrow for the list and the details side by
    /// side, the details then replace the list.
    narrow: bool,
    revealing: Option<Reveal>,
}

#[derive(Clone, Debug)]
struct Open {
    id: String,
}

/// The row to focus once it shows. The palette knows only the title it
/// searched for, closed details know their app among others of its title.
#[derive(Clone, Debug)]
enum Reveal {
    Titled(String),
    App(String),
}

impl Reveal {
    fn finds(&self, row: &AppRow) -> bool {
        match self {
            Self::Titled(title) => row.title == *title,
            Self::App(id) => row.id == *id,
        }
    }
}

type RowsKey = (AppOrder, Same<Snapshot>, u64, Look);

/// The chips' texts come from the search, the system, the queue and the
/// look, never from the picked category or the order.
type ChipsKey = (Same<Snapshot>, u64, Look);

/// The details of an app come from the system, the queue and the look.
type DetailKey = (Same<Snapshot>, u64, Look);

impl PageView for DiscoverPage {
    /// The list and the details each scroll on their own, so the page body
    /// has no scroll area of its own.
    fn shape(&self) -> BodyShape {
        BodyShape::Panes {
            width: f32::INFINITY,
        }
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        self.title(ui, listed(context));
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        // Escape closes the details like Back does. A dialog over the page,
        // such as the command palette, is drawn after it and answers Escape
        // itself.
        let dialog = ui.memory(|memory| memory.top_modal_layer().is_some());
        if self.open.is_some()
            && !dialog
            && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
        {
            self.close();
        }

        self.narrow = columns::is_narrow(ui.available_width());
        self.body(ui, context);
    }
}

impl DiscoverPage {
    pub fn reveal(&mut self, title: &str) {
        title.clone_into(&mut self.search);
        self.category = None;
        self.revealing = Some(Reveal::Titled(title.to_owned()));
    }

    fn open(&mut self, opened: Open) {
        if self.open.as_ref().is_some_and(|open| open.id == opened.id) {
            return;
        }

        self.open = Some(opened);
    }

    /// Closes the details. The list then shows the app's row focused again.
    fn close(&mut self) {
        if let Some(open) = self.open.take() {
            self.revealing = Some(Reveal::App(open.id));
        }
    }

    /// Search and sort show off until the apps are listed.
    fn title(&mut self, ui: &mut Ui, listed: bool) {
        title_header(ui, "page-discover", "page-discover-subtitle");
        page_toolbar(
            ui,
            self,
            |page, ui| {
                ui.enabled_ui(listed, |ui| {
                    let field = &mut page.search;
                    _ = search(ui, field, "discover-search", "discover-search-hint");
                });
            },
            |page, ui| {
                ui.enabled_ui(listed, |ui| page.sort(ui));
            },
        );
    }

    fn sort(&mut self, ui: &mut Ui) {
        let orders = [
            (AppOrder::Name, "discover-sort-name"),
            (AppOrder::Selected, "discover-sort-selected"),
        ];

        sort_select(ui, "discover-sort", &orders, &mut self.order);
    }

    fn body(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("discover ui").entered();
        let ctx = ui.ctx().clone();
        plan_running(ui, context);

        let catalogs = &context.catalogs.apps;
        notices(ui, context, catalogs.problem.as_ref());

        let apps = &catalogs.apps;
        let chips_rest = (Same::of(context.snapshot), context.revision, Look::of(ui));
        let chips = self.chips.get(&[&self.search], chips_rest, || {
            let _build = tracing::trace_span!("build chips").entered();
            let categories =
                apps.categories_matching(&self.search, context.snapshot, context.changes);

            Chips::of(&ctx, &categories)
        });

        let category = view::category(chips.ids(), self.category.as_deref());
        ui.enabled_ui(listed(context), |ui| self.filters(ui, &chips, category));
        ui.add_space(ui.tokens().sizes.header_gap);

        let beside = self.open.is_some();
        Columns::new("discover-columns").open(beside).show(
            ui,
            self,
            |ui, page| page.list(ui, context, &chips, category),
            |ui, page| details::body(page, ui, context),
        );
    }

    /// The apps under the chips and the search, each category under its
    /// heading, or the state that stands in for them.
    fn list(
        &mut self,
        ui: &mut Ui,
        context: &PageContext<'_>,
        chips: &Chips,
        category: Option<&str>,
    ) {
        let look = Look::of(ui);
        let apps = &context.catalogs.apps.apps;
        let query = view::Query {
            category,
            search: &self.search,
            order: self.order,
        };

        let rest = (
            self.order,
            Same::of(context.snapshot),
            context.revision,
            look,
        );

        let groups = self
            .rows
            .get(&[category.unwrap_or_default(), &self.search], rest, || {
                let _build = tracing::trace_span!("build rows").entered();
                let marked = self.search.trim();
                view::groups(apps, query, context.snapshot, context.changes)
                    .iter()
                    .map(|group| AppGroup::of(ui, group, marked))
                    .collect()
            });

        if groups.is_empty() {
            let elsewhere = category
                .filter(|_| chips.matches > 0)
                .and_then(|id| chips.title(id))
                .map(|title| (title.to_owned(), chips.matches));

            self.nothing(ui, elsewhere);
            return;
        }

        let selected = self.open.as_ref().map(|open| open.id.as_str());
        let (holds, opened) = show_groups(ui, context, &groups, self.revealing.as_ref(), selected);

        if holds {
            self.revealing = None;
        }

        if let Some(opened) = opened {
            self.open(opened);
        }
    }

    /// The category chips. They count the matches while a search runs and
    /// still pick a category.
    fn filters(&mut self, ui: &mut Ui, chips: &Chips, current: Option<&str>) {
        let categories = chips.categories.iter().map(|chip| {
            (
                Some(chip.id.as_str()),
                chip.title.as_str(),
                Some(chip.count.as_str()),
            )
        });

        let options: Vec<(Option<&str>, &str, Option<&str>)> =
            std::iter::once((None, chips.all.as_str(), Some(chips.total.as_str())))
                .chain(categories)
                .collect();

        // A second click on the picked category shows them all again, as
        // on the toggle pages.
        if let Some(picked) = counted_chips(ui, "discover-categories", &options, current).value {
            self.category = picked
                .filter(|picked| current != Some(*picked))
                .map(str::to_owned);
        }
    }

    /// The empty state. While a picked category hides the matches of
    /// others, `elsewhere` names it and how many apps match in all.
    fn nothing(&mut self, ui: &mut Ui, elsewhere: Option<(String, usize)>) {
        let ctx = ui.ctx().clone();
        let search = self.search.trim().to_owned();
        if let Some((category, matches)) = elsewhere.filter(|_| !search.is_empty()) {
            let named = [
                ("category", Arg::from(category.as_str())),
                ("search", Arg::from(search.as_str())),
            ];

            let title = i18n::text_with(&ctx, "discover-nothing-in-category", &named);

            let detail = i18n::text(&ctx, "discover-nothing-in-category-detail");
            let all = i18n::text_with(
                &ctx,
                "discover-show-all-matches",
                &[("count", Arg::from(matches))],
            );

            if empty_state_action(ui, icons::SYSTEM_SEARCH, &title, &detail, &all).clicked() {
                self.category = None;
            }

            return;
        }

        if search.is_empty() {
            let title = i18n::text(&ctx, "discover-none");
            let detail = i18n::text(&ctx, "discover-none-detail");
            empty_state(ui, icons::PACKAGE, &title, &detail);
            return;
        }

        let detail = i18n::text(&ctx, "discover-nothing-matches-detail");
        if search_empty_state(ui, &search, &detail) {
            self.search.clear();
        }
    }
}

/// Whether the Discover catalogs list at least one app.
fn listed(context: &PageContext<'_>) -> bool {
    context.catalogs.apps.apps.all().next().is_some()
}

/// A category's name in the user's language, else the catalog's English.
pub fn category_title(ctx: &egui::Context, id: &str, english: &str) -> String {
    i18n::text_or(ctx, &format!("discover-category-{id}"), english).into_owned()
}

/// Draws the categories under their headings. Returns whether the revealed
/// app is done with, so the page stops asking for it, and the app whose row
/// asked for its details. Beside the details the rows are one line tall.
fn show_groups(
    ui: &mut Ui,
    context: &PageContext<'_>,
    groups: &[AppGroup],
    revealing: Option<&Reveal>,
    selected: Option<&str>,
) -> (bool, Option<Open>) {
    let beside = selected.is_some();

    // A description of one line makes a shorter row than one of two, so
    // each row is measured and keeps the card row's padding.
    let heights = if beside {
        RowHeights::Fixed(row::compact_height(ui))
    } else {
        RowHeights::Measured {
            estimate: row::height(ui),
        }
    };

    let _rows = tracing::trace_span!("rows").entered();
    let revealed = revealing.and_then(|reveal| {
        groups
            .iter()
            .flat_map(|group| &group.rows)
            .find(|row| reveal.finds(row))
            .map(|row| row.id.clone())
    });

    let start = ui.cursor().top();
    let mut marks = Vec::with_capacity(groups.len());
    let mut holds = false;
    let mut opened = None;
    for group in groups {
        let top = ui.cursor().top() - start;
        section_heading(ui, &group.title, &group.count);
        let heading_bottom = ui.cursor().top() - start;
        ui.add_space(ui.tokens().sizes.heading_gap);

        let reveal = revealed
            .as_deref()
            .filter(|id| group.rows.iter().any(|row| row.id == *id));

        holds |= RowList::new(&group.id, &group.title, heights)
            .reveal(reveal)
            .selected(selected)
            .show(ui, group.rows.as_slice(), |ui, app, state| {
                dimmed(ui, context.busy, |ui| {
                    let asked = row::show(ui, context, app, state, beside);
                    if asked && opened.is_none() {
                        opened = Some(Open { id: app.id.clone() });
                    }
                });
            });

        marks.push(sticky::Mark {
            top,
            heading_bottom,
            end: ui.cursor().top() - start,
        });

        ui.add_space(ui.tokens().sizes.section_gap);
    }

    sticky::paint(ui, groups, &marks, start);

    // Asked again until the row holds the focus. An app the list does
    // not show is not waited for.
    (holds || revealed.is_none(), opened)
}
