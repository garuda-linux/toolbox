//! One app as its row shows it, built when the list changes and kept
//! until it changes again, so a frame builds no text.

use std::borrow::Cow;
use std::sync::Arc;

use egui::Ui;
use egui::text::{LayoutJob, TextWrapping};
use toolbox_core::features::discover::package;
use toolbox_gui_kit::fonts::{self, BODY, SMALL, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{ListItem, highlighted};
use toolbox_plan::PackageName;

use super::category_title;
use super::view::Shown;
use crate::i18n;
use crate::pages::rows::actions::{CopyName, Named, OpenWebsite, RowKind, Section, Website};

pub const DESCRIPTION_LINES: u8 = 2;

#[derive(Debug)]
pub struct AppRow {
    pub id: String,
    pub title: String,
    /// What screen readers say for the row: the title and what waits.
    pub name: String,
    pub title_job: Arc<LayoutJob>,
    pub description: Arc<LayoutJob>,
    /// The category's tag while a search lists every category.
    pub category: Option<String>,
    pub queued: Option<Cow<'static, str>>,
    /// Why the protected catalog keeps the installed app.
    pub kept: Option<Cow<'static, str>>,
    pub on: bool,
    pub url: Option<String>,
    pub package: Option<PackageName>,
    pub menu: String,
}

pub struct AppRows;

impl RowKind for AppRows {
    type Item = AppRow;

    const SECTIONS: &'static [Section<AppRow>] = &[Section {
        heading: None,
        actions: &[&OpenWebsite, &CopyName],
    }];
}

impl AppRow {
    pub fn of(ui: &Ui, shown: &Shown<'_>, query: &str) -> Self {
        let ctx = ui.ctx();
        let tokens = tokens::current(ctx);
        let hit = shown.hit;
        let entry = hit.entry;
        let title = entry.title().to_owned();
        let strong = fonts::font(ui.style(), Weight::SemiBold, BODY);
        let title_job = highlighted(ui, &title, query, strong, tokens.text);

        let small = fonts::font(ui.style(), Weight::Regular, SMALL);
        let mut description = highlighted(ui, entry.description(), query, small, tokens.secondary);
        description.wrap = TextWrapping {
            max_rows: usize::from(DESCRIPTION_LINES),
            break_anywhere: false,
            ..TextWrapping::default()
        };

        let queued = shown.state.queued.map(|wish| {
            i18n::text(
                ctx,
                if wish.is_on() {
                    "toggles-turns-on"
                } else {
                    "toggles-turns-off"
                },
            )
        });

        let kept = shown
            .state
            .kept
            .map(|why| i18n::text(ctx, &format!("protected-{}", why.as_str())));

        let name = queued
            .as_ref()
            .or(kept.as_ref())
            .map_or_else(|| title.clone(), |note| format!("{title}, {note}"));

        Self {
            id: entry.id().to_owned(),
            name,
            title_job: Arc::new(title_job),
            description: Arc::new(description),
            category: (!query.is_empty())
                .then(|| category_title(ctx, hit.category.id(), hit.category.title())),
            queued,
            kept,
            on: shown.state.shown_on(),
            url: entry.url().map(str::to_owned),
            package: package(entry).cloned(),
            menu: i18n::text_with(ctx, "discover-actions", &[("name", &title)]).into_owned(),
            title,
        }
    }
}

impl ListItem for AppRow {
    fn key(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for AppRow {
    fn copied_name(&self) -> &str {
        &self.title
    }
}

impl Website for AppRow {
    fn website(&self) -> Option<&str> {
        self.url.as_deref()
    }
}
