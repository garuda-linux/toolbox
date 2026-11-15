//! The Services page: system or user units, live from systemd, with a
//! search, filters and an order. Start, stop and the journal run at once
//! in the Console, enabling and masking wait in the queue.

mod ask;
mod row;
#[cfg(test)]
mod tests;
mod texts;
mod unit_row;

use std::sync::Arc;

use egui::Ui;
use toolbox_core::features::services::{Counts, Filter, Shown, UnitInfo, UnitOrder, counts, view};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    ButtonKind, RowHeights, RowList, Tone, after_skeleton, banner, card, counted_chips, dimmed,
    empty_state, error_row, hold_content, page_toolbar, pill_button, skeleton_rows,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::UnitScope;
use unit_row::UnitRow;

use super::header::{TabText, TabbedIds, search, tabbed_header};
use super::rows::{Look, sort_select};
use super::{PageContext, PageView, plan_running};
use crate::command::{AppCommand, ServiceCommand};
use crate::i18n;
use crate::page::PageId;
use crate::units::{Units, UnitsFailure};

/// Placeholder rows while the first list comes, enough to fill a window.
const SKELETON_ROWS: usize = 6;

#[derive(Debug)]
pub struct ServicesPage {
    scope: UnitScope,
    filter: Filter,
    order: UnitOrder,
    rows: RowCache<RowsKey, Listed>,
}

/// What the list shows for one filter, search and order.
#[derive(Debug)]
struct Listed {
    rows: Vec<UnitRow>,
    counts: Counts,
}

/// What the rows come from besides the search: the picked chip, the
/// order, the scope, the list, the queue and the look.
type RowsKey = (Shown, UnitOrder, UnitScope, Same<Vec<UnitInfo>>, u64, Look);

impl TabText for UnitScope {
    fn text_id(self) -> &'static str {
        match self {
            Self::System => "services-system",
            Self::User => "services-user",
        }
    }
}

impl Default for ServicesPage {
    fn default() -> Self {
        Self {
            scope: UnitScope::System,
            filter: Filter::default(),
            order: UnitOrder::Name,
            rows: RowCache::default(),
        }
    }
}

impl PageView for ServicesPage {
    fn left(&mut self, ctx: &egui::Context) {
        ask::forget(ctx);
    }

    fn place(&self) -> u64 {
        super::place_of(&self.scope)
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        self.title(ui, context);
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("services ui").entered();
        ask::show(ui.ctx(), context.commands);
        plan_running(ui, context);

        refused(ui, context);

        let ctx = ui.ctx().clone();
        let height = row::height(ui);
        let listed = context.service::<Units>().get(self.scope);
        let first_load = ui.make_persistent_id(("services-content", self.scope));
        let Some(units) = listed.latest() else {
            self.filters(ui, None);
            ui.add_space(ui.tokens().sizes.header_gap);
            if let Some(failure) = listed.error() {
                let retry = i18n::text(&ctx, "services-retry");
                if error_row(ui, &i18n::text(&ctx, failure_text(*failure)), &retry) {
                    context.commands.send(ServiceCommand::Refresh(self.scope));
                }
            } else {
                let name = i18n::text(&ctx, "services-reading");
                hold_content(&ctx, first_load);
                card(ui, |ui| skeleton_rows(ui, SKELETON_ROWS, height, &name));
            }

            return;
        };

        let built = self.built(ui, context, units);
        self.filters(ui, Some(built.counts));
        ui.add_space(ui.tokens().sizes.header_gap);
        if let Some(failure) = listed.error() {
            let again = i18n::text(&ctx, "services-refresh");
            let said = i18n::text(&ctx, failure_text(*failure));
            if banner(ui, Tone::Error, &said, &[&again]) == Some(0) {
                context.commands.send(ServiceCommand::Refresh(self.scope));
            }

            ui.add_space(ui.tokens().sizes.header_gap);
        }

        after_skeleton(ui, first_load, |ui| self.list(ui, context, &built, height));
    }
}

impl ServicesPage {
    /// Lists the units of `scope`, as the Settings page asks.
    pub const fn show_scope(&mut self, scope: UnitScope) {
        self.scope = scope;
    }

    /// The rows the filter lets through and the chips' counts, from the
    /// cache while nothing they come from changed.
    fn built(
        &mut self,
        ui: &Ui,
        context: &PageContext<'_>,
        units: &Arc<Vec<UnitInfo>>,
    ) -> Arc<Listed> {
        let rest = (
            self.filter.shown,
            self.order,
            self.scope,
            Same::of(units),
            context.revision,
            Look::of(ui),
        );

        self.rows.get(&[&self.filter.search], rest, || {
            let _build = tracing::trace_span!("build rows").entered();
            let query = self.filter.search.trim();
            let rows = view::rows(units, &self.filter, self.order, self.scope, context.changes)
                .iter()
                .map(|row| UnitRow::of(ui, row, self.scope, query))
                .collect();

            Listed {
                rows,
                counts: counts(units, &self.filter.search),
            }
        })
    }

    fn list(&mut self, ui: &mut Ui, context: &PageContext<'_>, built: &Listed, height: f32) {
        let ctx = ui.ctx().clone();
        if built.rows.is_empty() {
            self.nothing_listed(ui, built.counts);
            return;
        }

        let list = i18n::text(&ctx, "page-services");
        let _rows = tracing::trace_span!("rows").entered();
        RowList::new(("units", self.scope), &list, RowHeights::Fixed(height)).show(
            ui,
            built.rows.as_slice(),
            |ui, row, state| dimmed(ui, context.busy, |ui| row::show(ui, context, row, state)),
        );
    }

    /// The search finds units the picked chip hides: one click shows
    /// them, the search stays.
    fn nothing_listed(&mut self, ui: &mut Ui, counts: Counts) {
        let ctx = ui.ctx().clone();
        empty_state(
            ui,
            icons::SYSTEM_SEARCH,
            &i18n::text(&ctx, "services-none"),
            &i18n::text(&ctx, "services-none-detail"),
        );

        if self.filter.shown == Shown::All || counts.all == 0 {
            return;
        }

        let every = i18n::text_with(
            &ctx,
            "services-show-every",
            &[("count", &counts.all.to_string())],
        );

        ui.add_space(ui.tokens().sizes.control_gap);
        let clicked = ui
            .vertical_centered(|ui| pill_button(ui, &every, ButtonKind::Soft).clicked())
            .inner;

        if clicked {
            self.filter.shown = Shown::All;
        }
    }

    fn title(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ids = TabbedIds {
            title: "page-services",
            subtitle: "page-services-subtitle",
            tabs: "services-scope",
            refresh: "services-refresh",
        };

        let scopes = [UnitScope::System, UnitScope::User];
        let done = tabbed_header(ui, ids, &scopes, self.scope);
        if let Some(picked) = done.picked {
            self.scope = picked;
        }

        if done.refresh {
            context.commands.send(ServiceCommand::Refresh(self.scope));
        }

        page_toolbar(
            ui,
            self,
            |page, ui| {
                let field = &mut page.filter.search;
                _ = search(ui, field, "services-search", "services-search-hint");
            },
            Self::sort,
        );
    }

    fn sort(&mut self, ui: &mut Ui) {
        let orders = [
            (UnitOrder::Name, "services-sort-name"),
            (UnitOrder::State, "services-sort-state"),
        ];

        sort_select(ui, "services-sort", &orders, &mut self.order);
    }

    /// One chip per choice, counted once the list was read.
    fn filters(&mut self, ui: &mut Ui, counts: Option<Counts>) {
        let ctx = ui.ctx().clone();
        let labels = Shown::EVERY.map(|shown| {
            let label = i18n::text(&ctx, &format!("services-show-{}", shown.as_str()));
            let count = counts.map(|known| known.of(shown).to_string());
            (shown, label, count)
        });

        let options: Vec<(Shown, &str, Option<&str>)> = labels
            .iter()
            .map(|(shown, label, count)| (*shown, &**label, count.as_deref()))
            .collect();

        if let Some(picked) = counted_chips(ui, &options, self.filter.shown) {
            self.filter.shown = picked;
        }
    }
}

/// A start, stop or journal that did not run, as another plan ran, until
/// dismissed.
fn refused(ui: &mut Ui, context: &PageContext<'_>) {
    if !context.service::<Units>().refused() {
        return;
    }

    let ctx = ui.ctx().clone();
    let busy = i18n::text(&ctx, "services-busy");
    let dismiss = i18n::text(&ctx, "services-dismiss");
    if banner(ui, Tone::Warning, &busy, &[&dismiss]) == Some(0) {
        context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Services));
    }

    ui.add_space(ui.tokens().sizes.header_gap);
}

const fn failure_text(failure: UnitsFailure) -> &'static str {
    match failure {
        UnitsFailure::Unavailable => "services-unavailable",
        UnitsFailure::Denied => "services-denied",
        UnitsFailure::Failed => "services-failed",
    }
}
