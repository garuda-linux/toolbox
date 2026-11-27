//! One app as its row shows it, built when the list changes and kept
//! until it changes again, so a frame builds no text.

use std::borrow::Cow;
use std::sync::Arc;

use egui::Ui;
use egui::text::LayoutJob;
use toolbox_core::features::discover::package;
use toolbox_core::features::discover::view::{Group, Shown};
use toolbox_core::features::toggles::shown::{self as toggled, Control};
use toolbox_gui_kit::fonts::{self, BODY, SMALL, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{ListItem, Tone, highlighted};
use toolbox_plan::PackageName;

use super::category_title;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::rows::actions::{CopyName, Named, OpenWebsite, RowKind, Section, Website};
use crate::pages::toggles::control as row_control;

#[derive(Debug)]
pub struct AppGroup {
    pub id: String,
    pub title: String,
    pub count: String,
    pub rows: Vec<AppRow>,
}

#[derive(Debug)]
pub struct AppRow {
    pub id: String,
    pub title: String,
    /// What screen readers say for the row: the title and what waits.
    pub name: String,
    pub title_job: Arc<LayoutJob>,
    pub description: Arc<LayoutJob>,
    /// The description on one line, for a row beside the details.
    pub summary: Arc<LayoutJob>,
    pub queued: Option<Cow<'static, str>>,
    /// Why the protected catalog keeps the installed app.
    pub kept: Option<Cow<'static, str>>,
    /// "Installed" for an installed app without a queued change.
    pub installed: Option<Cow<'static, str>>,
    pub control: Control,
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

impl AppGroup {
    pub fn of(ui: &Ui, group: &Group<'_>, query: &str) -> Self {
        let ctx = ui.ctx();
        let category = group.category;
        Self {
            id: category.id().to_owned(),
            title: category_title(ctx, category.id(), category.title()),
            count: group.rows.len().to_string(),
            rows: group
                .rows
                .iter()
                .map(|shown| AppRow::of(ui, shown, query))
                .collect(),
        }
    }
}

impl AppRow {
    pub fn of(ui: &Ui, shown: &Shown<'_>, query: &str) -> Self {
        let ctx = ui.ctx();
        let tokens = tokens::current(ctx);
        let entry = shown.hit.entry;
        let title = entry.title().to_owned();
        let strong = fonts::font(ui.style(), Weight::SemiBold, BODY);
        let title_job = highlighted(ui, &title, query, strong, tokens.text);

        let small = fonts::font(ui.style(), Weight::Regular, SMALL);
        let words = highlighted(ui, entry.description(), query, small, tokens.secondary);
        let summary = row_control::cut_to(words.clone(), 1);
        let description = row_control::cut(words);
        let control = toggled::control(entry.check(), shown.state);
        let queued = shown
            .state
            .queued
            .map(|wish| i18n::text(ctx, row_control::queued_text(wish, control)));

        let kept = shown
            .state
            .kept
            .map(|why| i18n::text(ctx, &format!("protected-{}", why.as_str())));

        let installed = (control == Control::Remove).then(|| i18n::text(ctx, "packages-installed"));

        let name = queued
            .as_ref()
            .or(kept.as_ref())
            .or(installed.as_ref())
            .map_or_else(|| title.clone(), |note| format!("{title}, {note}"));

        Self {
            id: entry.id().to_owned(),
            name,
            title_job: Arc::new(title_job),
            description: Arc::new(description),
            summary: Arc::new(summary),
            queued,
            kept,
            installed,
            control,
            on: shown.state.shown_on(),
            url: entry.url().map(str::to_owned),
            package: package(entry).cloned(),
            menu: i18n::text_with(ctx, "discover-actions", &[("name", Arg::from(&title))])
                .into_owned(),
            title,
        }
    }
}

impl AppRow {
    /// What waits for the app or keeps it, beside its name. A queued change
    /// comes first, then a kept app, then an installed one.
    pub fn note(&self) -> Option<(&str, Tone)> {
        match (&self.queued, &self.kept, &self.installed) {
            (Some(queued), _, _) => Some((queued, Tone::Accent)),
            (None, Some(kept), _) => Some((kept, Tone::Neutral)),
            (None, None, Some(installed)) => Some((installed, Tone::Success)),
            (None, None, None) => None,
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
