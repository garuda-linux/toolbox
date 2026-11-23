pub mod control;
mod empty;
mod entry;
mod lead;
mod listed;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use egui::Ui;
pub use entry::paint_glyph;
pub use lead::{LeadChip, LeadSections, NoLead};
use listed::{Listed, SectionChip, SectionRows};
use toolbox_catalog::LoadProblem;
use toolbox_core::Snapshot;
use toolbox_core::features::search::Needle;
use toolbox_core::features::toggles::Refusal;
use toolbox_core::features::toggles::shown::{self as view, Section, Tally};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{
    RowHeights, RowList, Tone, banner, counted_chips, dimmed, page_header, section_heading,
};

use super::header::search;
use super::rows::{Look, refusal_banner};
use super::{PageContext, plan_running};
use crate::artwork::Artwork;
use crate::catalogs::CatalogId;
use crate::command::AppCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::refused::Shown as _;
use crate::toggle_pages::{TogglePages, ToggleRefusal};

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
    leads: RowCache<LeadKey, Vec<LeadChip>>,
}

/// What the catalog's rows come from besides the search: the system, the
/// queue, whether it was read, the icon theme's pictures and the look.
type ListedKey = (Same<Snapshot>, u64, bool, Option<usize>, Look);

/// What the lead chips come from besides the search: the lead data, read
/// from the system and the queue alone, and the look.
type LeadKey = (Same<Snapshot>, u64, Look);

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
            leads: RowCache::new(),
        }
    }

    /// While a plan runs, the entries fade and their switches rest, and
    /// a banner says why.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn waiting_for_plans(mut self) -> Self {
        self.waits = true;
        self
    }

    /// The search sits in the toolbar at the right, where KDE apps put it,
    /// and waits for the catalog's list, as the chips do.
    pub fn header(&mut self, ui: &mut Ui, listed: bool) {
        let ctx = ui.ctx().clone();
        let width = ui.tokens().sizes.search_width;
        let text = &mut self.search;
        page_header(
            ui,
            &i18n::text(&ctx, self.title),
            &i18n::text(&ctx, self.subtitle),
            width,
            |ui| {
                ui.enabled_ui(listed, |ui| {
                    _ = search(ui, text, "toggles-search", "toggles-search-hint");
                });
            },
        );
    }

    pub fn ui<L>(&mut self, ui: &mut Ui, context: &PageContext<'_>, (lead, data): (&L, &L::Data))
    where
        L: LeadSections,
    {
        let _ui = tracing::trace_span!("toggles ui").entered();
        let ctx = ui.ctx().clone();
        if self.waits {
            plan_running(ui, context);
        }

        let query = Needle::new(&self.search);
        let listed = self.listed(ui, context, &query);
        let rest = (Same::of(context.snapshot), context.revision, Look::of(ui));
        let leads = self
            .leads
            .get(&[query.as_str()], rest, || lead.chips(&ctx, data, &query));

        self.notices(ui, context, listed.unknown + lead.unread(data));
        let ready = context.catalogs.get(self.catalog).catalog.is_ok();
        ui.enabled_ui(ready, |ui| self.chips(ui, &leads, &listed.chips));
        ui.add_space(ui.tokens().sizes.filters_gap);

        let mut any = false;
        for chip in &*leads {
            if self
                .section
                .as_deref()
                .is_none_or(|picked| picked == chip.id)
            {
                any |= lead.show(ui, (context, data), chip.id, &query);
            }
        }

        for section in &listed.sections {
            any |= self.section(ui, context, section, true);
        }

        if !any {
            let hidden = empty::elsewhere(self.section.as_deref(), &leads, &listed.chips);
            let search = self.search.trim().to_owned();
            match empty::show(ui, &search, hidden.as_ref()) {
                Some(empty::Clear::Section) => self.section = None,
                Some(empty::Clear::Search) => self.search.clear(),
                None => {}
            }
        }
    }

    /// The catalog's sections, shared with the cache, so a frame that asks
    /// twice copies no title.
    pub fn steps(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> Steps {
        Steps(self.listed(ui, context, &Needle::default()))
    }

    /// The notices and the rows of the section `id` alone, without its
    /// heading, which the wizard's step title replaces.
    pub fn show_section(&mut self, ui: &mut Ui, context: &PageContext<'_>, id: &str) -> bool {
        let listed = self.listed(ui, context, &Needle::default());
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
    fn listed(&mut self, ui: &mut Ui, context: &PageContext<'_>, query: &Needle) -> Arc<Listed> {
        let catalog = match &context.catalogs.get(self.catalog).catalog {
            Ok(catalog) => catalog,
            Err(problem) => {
                banner(ui, Tone::Warning, &problem_text(ui.ctx(), problem), &[]);
                ui.add_space(ui.tokens().sizes.card_gap);
                return Arc::default();
            }
        };

        let icons = context
            .service::<Artwork>()
            .app_icons()
            .map(|icons| std::ptr::from_ref(icons).addr());

        let rest = (
            Same::of(context.snapshot),
            context.revision,
            context.probed,
            icons,
            Look::of(ui),
        );

        self.listed.get(&[query.as_str()], rest, || {
            let _build = tracing::trace_span!("build rows").entered();
            let shown =
                view::sections(catalog, context.snapshot, context.changes, context.desktops);

            Listed::of(ui, context, self.catalog, &shown, query)
        })
    }

    fn notices(&self, ui: &mut Ui, context: &PageContext<'_>, unknown: usize) {
        let ctx = ui.ctx().clone();
        let refused = context
            .service::<TogglePages>()
            .refusal(self.catalog)
            .and_then(|refused| refusal_text(&ctx, refused));

        if let Some(text) = refused
            && refusal_banner(ui, Tone::Warning, &text, "toggles-dismiss")
        {
            context
                .commands
                .send(AppCommand::DismissRefusal(self.catalog.page()));
        }

        if context.probed && unknown > 0 {
            let count = unknown;
            let text = i18n::text_with(&ctx, "toggles-unknown", &[("count", Arg::from(count))]);
            let again = i18n::text(&ctx, "common-read-again");
            if banner(ui, Tone::Warning, &text, &[&again]).value.is_some() {
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

        let read = sections.iter().all(|chip| chip.read) && leads.iter().all(|lead| lead.read);
        let all_count = shown_count(&ctx, tally, read);
        let leads = leads
            .iter()
            .map(|lead| (Some(lead.id), &*lead.title, lead.count.as_deref()));

        let sections = sections.iter().map(|chip| {
            (
                Some(chip.id.as_str()),
                chip.title.as_str(),
                Some(chip.count.as_str()),
            )
        });

        let options: Vec<(Option<&str>, &str, Option<&str>)> =
            std::iter::once((None, &*all, all_count.as_deref()))
                .chain(leads)
                .chain(sections)
                .collect();

        let current = self.section.as_deref();
        if let Some(picked) = counted_chips(ui, "toggles-sections", &options, current).value {
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
            section_heading(ui, &section.title, &section.count);
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
}

#[derive(Clone, Debug, Default)]
pub struct Steps(Arc<Listed>);

impl Steps {
    pub fn is_empty(&self) -> bool {
        self.0.chips.is_empty()
    }

    pub fn pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .chips
            .iter()
            .map(|chip| (chip.id.as_str(), chip.title.as_str()))
    }

    pub fn len(&self) -> usize {
        self.0.chips.len()
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
    i18n::text_with(
        ctx,
        "toggles-problem-reason",
        &[("reason", Arg::from(&reason))],
    )
    .into_owned()
}

/// "on/total", the count of a chip and of a section's heading.
pub fn tally_text(ctx: &egui::Context, tally: Tally) -> String {
    i18n::text_with(
        ctx,
        "toggles-section-count",
        &[
            ("on", Arg::from(tally.on)),
            ("total", Arg::from(tally.total)),
        ],
    )
    .into_owned()
}

/// [`tally_text`], or "?/total" while a state it counts is unread, since
/// how many are on is not known then.
pub fn count_text(ctx: &egui::Context, tally: Tally, read: bool) -> String {
    if read {
        return tally_text(ctx, tally);
    }

    let total = tally.total;
    i18n::text_with(
        ctx,
        "toggles-section-count-unread",
        &[("total", Arg::from(total))],
    )
    .into_owned()
}

/// [`count_text`], or none while nothing countable shows, since "0/0"
/// counts nothing.
pub fn shown_count(ctx: &egui::Context, tally: Tally, read: bool) -> Option<String> {
    (tally.total > 0).then(|| count_text(ctx, tally, read))
}

/// `None` for a stale id or an unmet entry, whose switch already rests.
pub fn refusal_text(ctx: &egui::Context, refused: &ToggleRefusal) -> Option<String> {
    match refused.refusal {
        Refusal::Account(_)
        | Refusal::Kernel(_)
        | Refusal::Protected(_)
        | Refusal::TakesAlong(_) => Some(refused.text(ctx)),
        Refusal::Unknown | Refusal::Unmet => None,
    }
}
