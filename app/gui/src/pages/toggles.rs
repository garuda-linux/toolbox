mod entry;
mod lead;
mod listed;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::Ui;
pub use lead::{LeadChip, LeadSections, NoLead};
use listed::{Listed, SectionChip, SectionRows};
use toolbox_catalog::LoadProblem;
use toolbox_core::Snapshot;
use toolbox_core::features::toggles::Refusal;
use toolbox_core::features::toggles::shown::{self as view, Section, Tally};
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    ButtonKind, RowHeights, RowList, Tone, banner, counted_chips, dimmed, empty_state,
    page_toolbar, pill_button, section_heading,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::header::{search, title_header};
use super::rows::{Look, refusal_banner};
use super::{PageContext, plan_running};
use crate::catalogs::CatalogId;
use crate::command::AppCommand;
use crate::i18n;

#[derive(Debug)]
pub struct TogglePage {
    catalog: CatalogId,
    title: &'static str,
    subtitle: &'static str,
    search: String,
    section: Option<String>,
    /// Its rows rest while a plan runs, see [`TogglePage::waiting_for_plans`].
    waits: bool,
    listed: RowCache<ListedKey, Listed>,
}

/// What the catalog's rows come from besides the search: the system, the
/// queue, whether it was read, the icon theme's pictures and the look.
type ListedKey = (Same<Snapshot>, u64, bool, Option<usize>, Look);

impl TogglePage {
    pub const fn new(catalog: CatalogId, title: &'static str, subtitle: &'static str) -> Self {
        Self {
            catalog,
            title,
            subtitle,
            search: String::new(),
            section: None,
            waits: false,
            listed: RowCache::new(),
        }
    }

    /// While a plan runs, the entries fade and their switches rest, and
    /// a banner says why.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn waiting_for_plans(mut self) -> Self {
        self.waits = true;
        self
    }

    pub fn header(&mut self, ui: &mut Ui) {
        title_header(ui, self.title, self.subtitle);
        page_toolbar(
            ui,
            &mut self.search,
            |text, ui| _ = search(ui, text, "toggles-search", "toggles-search-hint"),
            |_, _| {},
        );
    }

    pub fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>, lead: &dyn LeadSections) {
        let _ui = tracing::trace_span!("toggles ui").entered();
        let ctx = ui.ctx().clone();
        if self.waits {
            plan_running(ui, context);
        }

        let query = self.search.trim().to_lowercase();
        let listed = self.listed(ui, context, &query);
        let leads = lead.chips(&ctx, context, &query);
        self.notices(ui, context, listed.unknown);
        self.chips(ui, &leads, &listed.chips);
        ui.add_space(ui.tokens().sizes.filters_gap);

        let mut any = false;
        for chip in &leads {
            if self
                .section
                .as_deref()
                .is_none_or(|picked| picked == chip.id)
            {
                any |= lead.show(ui, context, chip.id, &query);
            }
        }

        for section in &listed.sections {
            any |= self.section(ui, context, section, true);
        }

        if !any {
            self.nothing_matches(ui);
        }
    }

    pub fn steps(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> Vec<(String, String)> {
        self.listed(ui, context, "")
            .chips
            .iter()
            .map(|chip| (chip.id.clone(), chip.title.clone()))
            .collect()
    }

    /// The notices and the rows of the section `id` alone, without its
    /// heading, which the wizard's step title replaces. Returns whether it
    /// had rows.
    pub fn show_section(&mut self, ui: &mut Ui, context: &PageContext<'_>, id: &str) -> bool {
        let listed = self.listed(ui, context, "");
        self.notices(ui, context, listed.unknown);
        listed
            .sections
            .iter()
            .find(|section| section.id == id)
            .is_some_and(|section| self.section(ui, context, section, false))
    }

    /// The catalog's sections that `query` matches, from the cache while
    /// nothing they come from changed. A broken catalog shows a banner
    /// and leaves the lead sections.
    fn listed(&mut self, ui: &mut Ui, context: &PageContext<'_>, query: &str) -> Arc<Listed> {
        let catalog = match &context.catalogs.get(self.catalog).catalog {
            Ok(catalog) => catalog,
            Err(problem) => {
                banner(ui, Tone::Warning, &problem_text(ui.ctx(), problem), &[]);
                ui.add_space(ui.tokens().sizes.card_gap);
                return Arc::default();
            }
        };

        let icons = context
            .app_icons
            .map(|icons| std::ptr::from_ref(icons).addr());

        let rest = (
            Same::of(context.snapshot),
            context.revision,
            context.probed,
            icons,
            Look::of(ui),
        );

        self.listed.get(&[query], rest, || {
            let _build = tracing::trace_span!("build rows").entered();
            let shown =
                view::sections(catalog, context.snapshot, context.changes, context.desktops);

            Listed::of(ui, context, self.catalog, &shown, query)
        })
    }

    fn notices(&self, ui: &mut Ui, context: &PageContext<'_>, unknown: usize) {
        let ctx = ui.ctx().clone();
        let refused = match context.refusals.catalogs.get(&self.catalog) {
            Some(refusal @ Refusal::Account(..)) => {
                let id = format!("account-refusal-{}", refusal.as_str());
                Some(i18n::text_with(&ctx, &id, &[("name", refusal.name())]))
            }
            Some(refusal @ Refusal::Kernel(..)) => {
                let id = format!("kernels-refusal-{}", refusal.as_str());
                Some(i18n::text_with(&ctx, &id, &[("name", refusal.name())]))
            }
            Some(refusal @ (Refusal::Protected(..) | Refusal::TakesAlong(..))) => {
                let id = if matches!(refusal, Refusal::TakesAlong(..)) {
                    "protected-along"
                } else {
                    "protected-refused"
                };

                let why = i18n::text(&ctx, &format!("protected-{}", refusal.as_str()));
                let args = [("name", refusal.name()), ("why", why.as_ref())];
                Some(i18n::text_with(&ctx, id, &args))
            }
            Some(Refusal::Unknown | Refusal::Unmet) | None => None,
        };

        if let Some(text) = refused
            && refusal_banner(ui, Tone::Warning, &text, "toggles-dismiss")
        {
            context
                .commands
                .send(AppCommand::DismissRefusal(self.catalog));
        }

        if context.probed && unknown > 0 {
            let count = unknown.to_string();
            let text = i18n::text_with(&ctx, "toggles-unknown", &[("count", &count)]);
            let again = i18n::text(&ctx, "common-read-again");
            if banner(ui, Tone::Warning, &text, &[&again]).is_some() {
                context.commands.send(AppCommand::ReadSystem);
            }

            ui.add_space(ui.tokens().sizes.card_gap);
        }
    }

    fn chips(&mut self, ui: &mut Ui, leads: &[LeadChip], sections: &[SectionChip]) {
        let ctx = ui.ctx().clone();
        let all = i18n::text(&ctx, "toggles-all");
        let tally = leads
            .iter()
            .map(|lead| lead.tally)
            .chain(sections.iter().map(|chip| chip.tally))
            .sum();

        let all_count = tally_text(&ctx, tally);
        let leads = leads
            .iter()
            .map(|lead| (Some(lead.id), &*lead.title, Some(lead.count.as_str())));

        let sections = sections.iter().map(|chip| {
            (
                Some(chip.id.as_str()),
                chip.title.as_str(),
                Some(chip.count.as_str()),
            )
        });

        let options: Vec<(Option<&str>, &str, Option<&str>)> =
            std::iter::once((None, &*all, Some(all_count.as_str())))
                .chain(leads)
                .chain(sections)
                .collect();

        // A second click on the chosen section shows them all again.
        let current = self.section.as_deref();
        if let Some(picked) = counted_chips(ui, &options, current) {
            self.section = picked
                .filter(|picked| current != Some(*picked))
                .map(str::to_owned);
        }
    }

    fn section(
        &self,
        ui: &mut Ui,
        context: &PageContext<'_>,
        section: &SectionRows,
        headed: bool,
    ) -> bool {
        if self
            .section
            .as_deref()
            .is_some_and(|picked| picked != section.id)
        {
            return false;
        }

        if headed {
            section_heading(ui, section.icon, &section.title, &section.count);
            ui.add_space(ui.tokens().sizes.heading_gap);
        }

        let place = entry::Place {
            catalog: self.catalog,
            busy: self.waits && context.busy,
        };

        let heights = RowHeights::Measured {
            estimate: entry::estimate(ui),
        };

        let _rows = tracing::trace_span!("rows").entered();
        RowList::new(&section.id, &section.title, heights).show(
            ui,
            &section.entries,
            |ui, row, state| {
                dimmed(ui, place.busy, |ui| {
                    entry::show(ui, context, place, row, state);
                });
            },
        );
        ui.add_space(ui.tokens().sizes.section_gap);

        true
    }

    fn nothing_matches(&mut self, ui: &mut Ui) {
        let search = self.search.trim().to_owned();
        if search.is_empty() {
            return;
        }

        let ctx = ui.ctx().clone();
        let detail = i18n::text_with(
            &ctx,
            "toggles-nothing-matches-detail",
            &[("search", &search)],
        );

        empty_state(
            ui,
            icons::SYSTEM_SEARCH,
            &i18n::text(&ctx, "toggles-nothing-matches"),
            &detail,
        );

        ui.add_space(ui.tokens().sizes.card_gap);
        ui.vertical_centered(|ui| {
            let clear = i18n::text(&ctx, "toggles-clear-search");
            if pill_button(ui, &clear, ButtonKind::Soft).clicked() {
                self.search.clear();
            }
        });
    }
}

/// A catalog text in the user's language, else the catalog's English.
pub fn catalog_text(ctx: &egui::Context, catalog: CatalogId, id: &str, english: &str) -> String {
    i18n::text_or(ctx, &format!("catalog-{}-{id}", catalog.as_str()), english).into_owned()
}

fn section_title(ctx: &egui::Context, catalog: CatalogId, section: &Section<'_>) -> String {
    catalog_text(ctx, catalog, section.section.id(), section.section.title())
}

fn problem_text(ctx: &egui::Context, problem: &LoadProblem) -> String {
    let reason = i18n::text(ctx, &format!("toggles-problem-{}", problem.code));
    format!("{} {reason}", i18n::text(ctx, "toggles-problem"))
}

/// "on/total", the count of a chip and of a section's heading.
pub fn tally_text(ctx: &egui::Context, tally: Tally) -> String {
    let (on, total) = (tally.on.to_string(), tally.total.to_string());
    i18n::text_with(
        ctx,
        "toggles-section-count",
        &[("on", &on), ("total", &total)],
    )
    .into_owned()
}
