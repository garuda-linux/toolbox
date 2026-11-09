//! The Services page: system or user units, live from systemd, with a
//! search, filters and an order. Start, stop and the journal run at once
//! in the Console, enabling and masking wait in the queue.

mod ask;
mod row;
#[cfg(test)]
mod tests;
mod texts;
mod unit_row;
pub mod view;

use std::sync::Arc;

use egui::Ui;
use toolbox_core::features::services::{Filter, UnitInfo, UnitOrder};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    RowHeights, RowList, after_skeleton, card, dimmed, empty_state, error_row, hold_content,
    page_toolbar, skeleton_rows, toggle_chips,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::UnitScope;
use unit_row::UnitRow;

use super::header::{TabText, TabbedIds, search, tabbed_header};
use super::rows::{Look, sort_select};
use super::{PageContext, PageView, plan_running};
use crate::command::ServiceCommand;
use crate::i18n;
use crate::units::UnitsFailure;

/// Placeholder rows while the first list comes, enough to fill a window.
const SKELETON_ROWS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shown {
    Disabled,
    Inactive,
}

#[derive(Debug)]
pub struct ServicesPage {
    scope: UnitScope,
    filter: Filter,
    order: UnitOrder,
    rows: RowCache<RowsKey, Vec<UnitRow>>,
}

/// What the rows come from besides the search: the two filters, the
/// order, the scope, the list, the queue and the look.
type RowsKey = (
    bool,
    bool,
    UnitOrder,
    UnitScope,
    Same<Vec<UnitInfo>>,
    u64,
    Look,
);

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
        self.filters(ui);
        ui.add_space(ui.tokens().sizes.header_gap);

        let ctx = ui.ctx().clone();
        let height = row::height(ui);
        let listed = context.units.get(self.scope);
        let first_load = ui.make_persistent_id(("services-content", self.scope));
        let units = match (listed.error(), listed.latest()) {
            (Some(failure), _) => {
                let id = match failure {
                    UnitsFailure::Unavailable => "services-unavailable",
                    UnitsFailure::Denied => "services-denied",
                    UnitsFailure::Failed => "services-failed",
                };

                let retry = i18n::text(&ctx, "services-retry");
                if error_row(ui, &i18n::text(&ctx, id), &retry) {
                    context.commands.send(ServiceCommand::Refresh(self.scope));
                }

                return;
            }
            (None, None) => {
                let name = i18n::text(&ctx, "services-reading");
                hold_content(&ctx, first_load);
                card(ui, |ui| skeleton_rows(ui, SKELETON_ROWS, height, &name));
                return;
            }
            (None, Some(units)) => units,
        };

        after_skeleton(ui, first_load, |ui| self.list(ui, context, units, height));
    }
}

impl ServicesPage {
    /// Lists the units of `scope`, as the Settings page asks.
    pub const fn show_scope(&mut self, scope: UnitScope) {
        self.scope = scope;
    }

    /// The units the filters let through, from the cache while nothing
    /// they come from changed.
    fn list(
        &mut self,
        ui: &mut Ui,
        context: &PageContext<'_>,
        units: &Arc<Vec<UnitInfo>>,
        height: f32,
    ) {
        let ctx = ui.ctx().clone();
        let rest = (
            self.filter.include_disabled,
            self.filter.hide_inactive,
            self.order,
            self.scope,
            Same::of(units),
            context.revision,
            Look::of(ui),
        );

        let rows = self.rows.get(&[&self.filter.search], rest, || {
            let _build = tracing::trace_span!("build rows").entered();
            let query = self.filter.search.trim();
            view::rows(units, &self.filter, self.order, self.scope, context.changes)
                .iter()
                .map(|row| UnitRow::of(ui, row, self.scope, query))
                .collect()
        });

        if rows.is_empty() {
            empty_state(
                ui,
                icons::SYSTEM_SEARCH,
                &i18n::text(&ctx, "services-none"),
                &i18n::text(&ctx, "services-none-detail"),
            );
            return;
        }

        let list = i18n::text(&ctx, "page-services");
        let _rows = tracing::trace_span!("rows").entered();
        RowList::new(("units", self.scope), &list, RowHeights::Fixed(height)).show(
            ui,
            &rows,
            |ui, row, state| dimmed(ui, context.busy, |ui| row::show(ui, context, row, state)),
        );
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

    fn filters(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let disabled = i18n::text(&ctx, "services-hide-disabled");
        let inactive = i18n::text(&ctx, "services-hide-inactive");
        let options = [(Shown::Disabled, &*disabled), (Shown::Inactive, &*inactive)];
        let filter = &self.filter;
        let on = |chip| match chip {
            Shown::Disabled => !filter.include_disabled,
            Shown::Inactive => filter.hide_inactive,
        };

        match toggle_chips(ui, &options, on) {
            Some(Shown::Disabled) => self.filter.include_disabled ^= true,
            Some(Shown::Inactive) => self.filter.hide_inactive ^= true,
            None => {}
        }
    }
}
