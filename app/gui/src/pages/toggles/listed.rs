//! What a toggle page lists of its catalog, built when the catalog's
//! state, the queue, the search or the look change and kept until then,
//! so a frame builds no text: the chips' texts, the sections that have a
//! match and their entries as rows.

use std::borrow::Cow;
use std::sync::Arc;

use egui::Ui;
use egui::text::LayoutJob;
use toolbox_core::features::search::Needle;
use toolbox_core::features::toggles::shown::{self, Control, Entry, Section, Tally};
use toolbox_gui_kit::fonts::{self, BODY, SMALL, Weight};
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{ListItem, Tone, highlighted};

use super::{catalog_text, control, count_text, entry, section_title};
use crate::catalogs::CatalogId;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::rows::actions::{
    CopyName, Named, OpenWebsite, RowKind, Section as Actions, Website,
};

#[derive(Debug, Default)]
pub struct Listed {
    pub chips: Vec<SectionChip>,
    pub unknown: usize,
    /// The sections with an entry the search matched.
    pub sections: Vec<SectionRows>,
}

/// A catalog section's chip, as [`super::LeadChip`] is a lead section's.
#[derive(Debug)]
pub struct SectionChip {
    pub id: String,
    pub title: String,
    pub tally: Tally,
    /// Every entry it counts shows a state.
    pub read: bool,
    /// Such as "2/4" beside the title, "?/4" while a state is unread.
    pub count: String,
}

#[derive(Debug)]
pub struct SectionRows {
    pub id: String,
    pub title: String,
    /// Such as "2/8".
    pub count: String,
    pub entries: Vec<EntryRow>,
}

#[derive(Debug)]
pub struct EntryRow {
    pub id: String,
    pub title: String,
    /// What screen readers say for the row: the title and its note.
    pub name: String,
    pub title_job: Arc<LayoutJob>,
    pub description: Arc<LayoutJob>,
    /// What the entry waits for, beside the title.
    pub note: Option<(Cow<'static, str>, Tone)>,
    /// A probe could not read what the entry checks.
    pub unknown: bool,
    pub control: Control,
    /// "Installed" or "Enabled" beside the title of an entry that is.
    pub status: Option<Cow<'static, str>>,
    pub on: bool,
    pub changeable: bool,
    pub url: Option<String>,
    /// The catalog's glyph, shown in place of a picture.
    pub glyph: Option<Icon>,
    /// The picture's address for egui's loaders.
    pub picture: Option<String>,
    /// What picks the hue of the letter tile shown without a picture.
    pub tile_package: String,
    pub menu: String,
}

pub struct EntryRows;

impl RowKind for EntryRows {
    type Item = EntryRow;

    const SECTIONS: &'static [Actions<EntryRow>] = &[Actions {
        heading: None,
        actions: &[&OpenWebsite, &CopyName],
    }];
}

impl Listed {
    /// The page's catalog part for `sections`, the entries `query`
    /// (trimmed, lower case) matches.
    pub fn of(
        ui: &Ui,
        context: &PageContext<'_>,
        catalog: CatalogId,
        sections: &[Section<'_>],
        query: &Needle,
    ) -> Self {
        let ctx = ui.ctx();
        let chips = sections
            .iter()
            .map(|section| {
                let matched = |entry: &Entry<'_>| entry::matches(ctx, catalog, entry, query);
                let tally = section.tally(matched);
                let read = section.read(matched);
                SectionChip {
                    id: section.section.id().to_owned(),
                    title: section_title(ctx, catalog, section),
                    tally,
                    read,
                    count: count_text(ctx, tally, read),
                }
            })
            .collect();

        let unknown = shown::unknown(sections);
        let sections = sections
            .iter()
            .filter_map(|section| SectionRows::of(ui, context, catalog, section, query))
            .collect();

        Self {
            chips,
            unknown,
            sections,
        }
    }
}

impl SectionRows {
    fn of(
        ui: &Ui,
        context: &PageContext<'_>,
        catalog: CatalogId,
        section: &Section<'_>,
        query: &Needle,
    ) -> Option<Self> {
        let ctx = ui.ctx();
        let entries: Vec<EntryRow> = section
            .entries
            .iter()
            .filter(|entry| entry::matches(ctx, catalog, entry, query))
            .map(|entry| EntryRow::of(ui, context, catalog, entry, query))
            .collect();

        if entries.is_empty() {
            return None;
        }

        let matched = |entry: &Entry<'_>| entry::matches(ctx, catalog, entry, query);
        Some(Self {
            id: section.section.id().to_owned(),
            title: section_title(ctx, catalog, section),
            count: count_text(ctx, section.tally(matched), section.read(matched)),
            entries,
        })
    }
}

impl EntryRow {
    fn of(
        ui: &Ui,
        context: &PageContext<'_>,
        catalog: CatalogId,
        entry: &Entry<'_>,
        query: &Needle,
    ) -> Self {
        let ctx = ui.ctx();
        let tokens = tokens::current(ctx);
        let title = catalog_text(ctx, catalog, entry.entry.id(), entry.entry.title());
        let ink = if entry.changeable() {
            tokens.text
        } else {
            tokens.dim
        };

        let strong = fonts::font(ui.style(), Weight::SemiBold, BODY);
        let title_job = highlighted(ui, &title, query.as_str(), strong, ink);
        let small = fonts::font(ui.style(), Weight::Regular, SMALL);
        let words = entry::description(ctx, catalog, entry);
        let description = control::cut(highlighted(
            ui,
            &words,
            query.as_str(),
            small,
            tokens.secondary,
        ));

        let control = control::of(catalog, entry);

        let note = entry::note(ctx, catalog, entry, control);
        let status = entry::status(catalog, entry, control).map(|text| i18n::text(ctx, text));

        let name = note
            .as_ref()
            .map(|(note, _)| note)
            .or(status.as_ref())
            .map_or_else(|| title.clone(), |note| format!("{title}, {note}"));

        let url = entry.entry.url().map(str::to_owned);
        Self {
            id: entry.entry.id().to_owned(),
            name,
            title_job: Arc::new(title_job),
            description: Arc::new(description),
            note,
            unknown: context.probed && entry.unread(),
            control,
            status,
            on: entry.state.shown_on(),
            changeable: entry.changeable(),
            glyph: entry.entry.glyph().and_then(icons::named),
            picture: entry::picture_file(context, catalog, entry)
                .map(|file| icons::picture_uri(&file)),
            tile_package: entry::tile_package(entry.entry).to_owned(),
            menu: i18n::text_with(ctx, "row-menu", &[("name", Arg::from(&title))]).into_owned(),
            url,
            title,
        }
    }
}

impl ListItem for EntryRow {
    fn key(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for EntryRow {
    fn copied_name(&self) -> &str {
        &self.title
    }
}

impl Website for EntryRow {
    fn website(&self) -> Option<&str> {
        self.url.as_deref()
    }
}
