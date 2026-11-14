mod app_row;
mod chips;
mod row;
#[cfg(test)]
mod tests;

use app_row::AppRow;
use chips::Chips;
use egui::Ui;
use toolbox_catalog::LoadProblem;
use toolbox_core::Snapshot;
use toolbox_core::domains::InstalledPackages;
use toolbox_core::features::discover::{AppOrder, view};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    ButtonKind, RowHeights, RowList, Tone, banner, counted_chips, dimmed, empty_state,
    page_toolbar, pill_button,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::header::{search, title_header};
use super::rows::{Look, refusal_banner, sort_select};
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
    rows: RowCache<RowsKey, Vec<AppRow>>,
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
    fn header(&mut self, ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        self.title(ui);
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

    fn title(&mut self, ui: &mut Ui) {
        title_header(ui, "page-discover", "page-discover-subtitle");
        page_toolbar(
            ui,
            self,
            |page, ui| {
                let field = &mut page.search;
                _ = search(ui, field, "discover-search", "discover-search-hint");
            },
            Self::sort,
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
        self.filters(ui, &chips, category);
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

        let rows = self
            .rows
            .get(&[category.unwrap_or_default(), &self.search], rest, || {
                let _build = tracing::trace_span!("build rows").entered();
                let marked = self.search.trim();
                view::rows(apps, query, context.snapshot, context.changes)
                    .iter()
                    .map(|shown| AppRow::of(ui, shown, marked))
                    .collect()
            });

        if rows.is_empty() {
            self.nothing(ui);
            return;
        }

        let list = i18n::text(&ctx, "page-discover");

        // A description of one line makes a shorter row than one of two, so
        // each row is measured and keeps the card row's padding.
        let heights = RowHeights::Measured {
            estimate: row::height(ui),
        };

        let _rows = tracing::trace_span!("rows").entered();
        let revealed = self.revealing.as_deref().and_then(|title| {
            rows.iter()
                .find(|row| row.title == title)
                .map(|row| row.id.clone())
        });

        let holds = RowList::new("apps", &list, heights)
            .reveal(revealed.as_deref())
            .show(ui, rows.as_slice(), |ui, app, state| {
                dimmed(ui, context.busy, |ui| row::show(ui, context, app, state));
            });

        // Asked again until the row holds the focus. An app the list does
        // not show is not waited for.
        if holds || revealed.is_none() {
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

        if let Some(picked) = counted_chips(ui, &options, current) {
            self.category = picked.map(str::to_owned);
        }
    }

    fn nothing(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let search = self.search.trim().to_owned();
        let (icon, title, detail) = if search.is_empty() {
            (
                icons::PACKAGE,
                i18n::text(&ctx, "discover-none"),
                i18n::text(&ctx, "discover-none-detail"),
            )
        } else {
            (
                icons::SYSTEM_SEARCH,
                i18n::text_with(&ctx, "discover-nothing-matches", &[("search", &search)]),
                i18n::text(&ctx, "discover-nothing-matches-detail"),
            )
        };

        empty_state(ui, icon, &title, &detail);
        if !search.is_empty() {
            ui.add_space(ui.tokens().sizes.card_gap);
            ui.vertical_centered(|ui| {
                let clear = i18n::text(&ctx, "discover-clear-search");
                if pill_button(ui, &clear, ButtonKind::Soft).clicked() {
                    self.search.clear();
                }
            });
        }
    }
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
        if banner(ui, Tone::Warning, &text, &[&again]).is_some() {
            context.commands.send(AppCommand::ReadSystem);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }
}

/// A category's name in the user's language, else the catalog's English.
pub fn category_title(ctx: &egui::Context, id: &str, english: &str) -> String {
    i18n::text_or(ctx, &format!("discover-category-{id}"), english).into_owned()
}
