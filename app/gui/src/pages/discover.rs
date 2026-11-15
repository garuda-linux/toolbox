mod app_row;
mod chips;
mod row;
mod sticky;
#[cfg(test)]
mod tests;

use app_row::AppGroup;
use chips::Chips;
use egui::Ui;
use toolbox_catalog::LoadProblem;
use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::discover::{AppOrder, view};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    RowHeights, RowList, Tone, banner, counted_chips, dimmed, empty_state, empty_state_action,
    page_toolbar, section_heading,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::header::{search_beside_sort, title_header};
use super::rows::{Look, refusal_banner, sort_select};
use super::search_empty::search_empty_state;
use super::toggles::refusal_text;
use super::{PageContext, PageView, plan_running};
use crate::command::AppCommand;
use crate::i18n;
use crate::page::PageId;
use crate::toggle_pages::TogglePages;

#[derive(Debug, Default)]
pub struct DiscoverPage {
    search: String,
    category: Option<String>,
    order: AppOrder,
    rows: RowCache<RowsKey, Vec<AppGroup>>,
    chips: RowCache<ChipsKey, Chips>,
    /// The title of an app picked in the palette, whose row takes the
    /// focus once it shows.
    revealing: Option<String>,
}

type RowsKey = (AppOrder, Same<Snapshot>, u64, Look);

/// The chips' texts come from the search, the system, the queue and the
/// look, never from the picked category or the order.
type ChipsKey = (Same<Snapshot>, u64, Look);

impl PageView for DiscoverPage {
    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        self.title(ui, listed(context));
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        self.body(ui, context);
    }
}

impl DiscoverPage {
    pub fn reveal(&mut self, title: &str) {
        title.clone_into(&mut self.search);
        self.category = None;
        self.revealing = Some(title.to_owned());
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
                    _ = search_beside_sort(ui, field, "discover-search", "discover-search-hint");
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
        let look = Look::of(ui);
        let chips_rest = (Same::of(context.snapshot), context.revision, look.clone());
        let chips = self.chips.get(&[&self.search], chips_rest, || {
            let _build = tracing::trace_span!("build chips").entered();
            let categories =
                apps.categories_matching(&self.search, context.snapshot, context.changes);

            Chips::of(&ctx, &categories)
        });

        let category = view::category(chips.ids(), self.category.as_deref());
        ui.enabled_ui(listed(context), |ui| self.filters(ui, &chips, category));
        ui.add_space(ui.tokens().sizes.header_gap);

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

        if show_groups(ui, context, &groups, self.revealing.as_deref()) {
            self.revealing = None;
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
            let count = matches.to_string();
            let named = [("category", category.as_str()), ("search", search.as_str())];
            let title = i18n::text_with(&ctx, "discover-nothing-in-category", &named);
            let detail = i18n::text(&ctx, "discover-nothing-in-category-detail");
            let all = i18n::text_with(&ctx, "discover-show-all-matches", &[("count", &count)]);
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

fn notices(ui: &mut Ui, context: &PageContext<'_>, problem: Option<&LoadProblem>) {
    let ctx = ui.ctx().clone();
    if let Some(problem) = problem {
        let reason = i18n::text(&ctx, &format!("toggles-problem-{}", problem.code));
        let text = format!("{} {reason}", i18n::text(&ctx, "discover-problem"));
        banner(ui, Tone::Warning, &text, &[]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }

    let refused = context
        .service::<TogglePages>()
        .app_refusal()
        .and_then(|refused| refusal_text(&ctx, refused));

    if let Some(text) = refused
        && refusal_banner(ui, Tone::Warning, &text, "toggles-dismiss")
    {
        context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Discover));
    }

    // Without the installed packages every switch shows off, and switching
    // an installed app on would queue nothing.
    if context.probed && context.snapshot.part::<InstalledPackages>().is_none() {
        let again = i18n::text(&ctx, "common-read-again");
        let text = i18n::text(&ctx, "discover-unreadable");
        if banner(ui, Tone::Warning, &text, &[&again]).value.is_some() {
            context.commands.send(AppCommand::ReadSystem);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }
}

/// A category's name in the user's language, else the catalog's English.
pub fn category_title(ctx: &egui::Context, id: &str, english: &str) -> String {
    i18n::text_or(ctx, &format!("discover-category-{id}"), english).into_owned()
}

/// Whether the revealed app is done with, so the page stops asking for it.
fn show_groups(
    ui: &mut Ui,
    context: &PageContext<'_>,
    groups: &[AppGroup],
    revealing: Option<&str>,
) -> bool {
    // A description of one line makes a shorter row than one of two, so
    // each row is measured and keeps the card row's padding.
    let heights = RowHeights::Measured {
        estimate: row::height(ui),
    };

    let _rows = tracing::trace_span!("rows").entered();
    let revealed = revealing.and_then(|title| {
        groups
            .iter()
            .flat_map(|group| &group.rows)
            .find(|row| row.title == title)
            .map(|row| row.id.clone())
    });

    let start = ui.cursor().top();
    let mut marks = Vec::with_capacity(groups.len());
    let mut holds = false;
    for group in groups {
        let top = ui.cursor().top() - start;
        section_heading(ui, group.icon, &group.title, &group.count);
        let heading_bottom = ui.cursor().top() - start;
        ui.add_space(ui.tokens().sizes.heading_gap);

        let reveal = revealed
            .as_deref()
            .filter(|id| group.rows.iter().any(|row| row.id == *id));

        holds |= RowList::new(&group.id, &group.title, heights)
            .reveal(reveal)
            .show(ui, group.rows.as_slice(), |ui, app, state| {
                dimmed(ui, context.busy, |ui| row::show(ui, context, app, state));
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
    holds || revealed.is_none()
}
