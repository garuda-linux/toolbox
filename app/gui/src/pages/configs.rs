//! Files of the user open in the desktop's app.
//! Root-owned files are edited as a private copy in the Console and
//! written back by the helper after the user saw what changed. A few are
//! only shown.

mod list;
mod review;
#[cfg(test)]
mod tests;
mod texts;
mod view;

use std::collections::BTreeSet;

use egui::Ui;
use toolbox_core::features::configs::session::{Phase, Stage};
use toolbox_core::features::configs::viewing;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::theme::PILL_HEIGHT;
use toolbox_gui_kit::widgets::{ButtonKind, Tone, banner, mono_small, page_toolbar, pill_button};

use super::header::{refresh_header_reading, search};
use super::rows::refused_page_banner;
use super::{PageContext, PageView};
use crate::command::ConfigsCommand;
use crate::configs::Configs;
use crate::i18n;
use crate::page::PageId;

const MIN_TEXT_HEIGHT: f32 = 120.0;

#[derive(Debug, Default)]
pub struct ConfigsPage {
    search: String,
    expanded: BTreeSet<String>,
    rows: list::Rows,
    /// The review dialog was closed, the changes still wait.
    review_closed: bool,
}

impl PageView for ConfigsPage {
    /// A list: the rows take the window's width, as Kirigami's list pages do.
    fn shape(&self) -> toolbox_gui_kit::widgets::BodyShape {
        toolbox_gui_kit::widgets::BodyShape::Wide
    }

    fn left(&mut self, ctx: &egui::Context) {
        review::forget(ctx);
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        let reading = context.service::<Configs>().reading();
        if refresh_header_reading(
            ui,
            ("page-configs", "page-configs-subtitle"),
            "configs-refresh",
            reading,
        ) {
            context.commands.send(ConfigsCommand::Refresh);
        }

        page_toolbar(
            ui,
            &mut self.search,
            |text, ui| _ = search(ui, text, "configs-search", "configs-search-hint"),
            |_, _| {},
        );

        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("configs ui").entered();
        refusal(ui, context);
        self.status(ui, context);
        list::show(
            ui,
            context,
            (&mut self.search, &mut self.expanded),
            &mut self.rows,
        );

        let ctx = ui.ctx().clone();
        let configs = context.service::<Configs>();
        let session = configs.session();
        match (session.phase() == Phase::Review, self.review_closed) {
            (false, _) => self.review_closed = false,
            (true, false) => self.review_closed = review::show(&ctx, context, session),
            (true, true) => {}
        }

        if configs.viewing().phase() != viewing::Phase::Idle {
            view::show(&ctx, context, configs.viewing());
        }
    }
}

impl ConfigsPage {
    fn status(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        let session = context.service::<Configs>().session();
        let Some(text) = texts::status(&ctx, session) else {
            return;
        };

        let phase = session.phase();
        let tone = match phase {
            Phase::Failed => Tone::Error,
            Phase::Saved => Tone::Success,
            Phase::Idle
            | Phase::Reading
            | Phase::Starting(_)
            | Phase::Running(_)
            | Phase::Comparing
            | Phase::Review
            | Phase::Unchanged => Tone::Neutral,
        };

        let finished = matches!(phase, Phase::Saved | Phase::Unchanged | Phase::Failed);
        let dismiss = i18n::text(&ctx, "configs-dismiss");
        let action = finished.then_some(dismiss.as_ref());
        if banner(ui, tone, &text, action.as_slice()).value == Some(0) {
            context.commands.send(ConfigsCommand::Dismiss);
        }

        if let (Phase::Failed, Some(detail)) = (phase, session.problem().and_then(texts::detail)) {
            ui.label(mono_small(ui, detail, ui.tokens().text));
        }

        let word = |id: &str| i18n::text(&ctx, id).into_owned();
        ui.horizontal(|ui| match phase {
            Phase::Starting(Stage::Editor) | Phase::Running(Stage::Editor) => {
                if pill_button(ui, &word("configs-show-console"), ButtonKind::Soft).clicked() {
                    context.commands.send(ConfigsCommand::ShowConsole);
                }

                if pill_button(ui, &word("configs-cancel-edit"), ButtonKind::Soft).clicked() {
                    context.commands.send(ConfigsCommand::Discard);
                }
            }
            Phase::Review if self.review_closed => {
                if pill_button(ui, &word("configs-review"), ButtonKind::Soft).clicked() {
                    self.review_closed = false;
                }
            }
            Phase::Idle
            | Phase::Reading
            | Phase::Starting(Stage::Write)
            | Phase::Running(Stage::Write)
            | Phase::Comparing
            | Phase::Review
            | Phase::Saved
            | Phase::Unchanged
            | Phase::Failed => {}
        });

        ui.add_space(ui.tokens().sizes.card_gap);
    }
}

fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let refused = context.service::<Configs>().refusal();
    refused_page_banner(ui, context, refused, PageId::Configs, "configs-dismiss");
}

/// A dialog's `body` above its `buttons`, together as tall as they need
/// up to the window's height, so a one-line error gets no empty page.
fn stacked<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui), buttons: impl FnOnce(&mut Ui) -> R) -> R {
    body(ui);
    ui.add_space(ui.tokens().sizes.dialog_gap);
    buttons(ui)
}

/// How tall a scrolled text in a [`stacked`] body may be: the room left
/// once the buttons below it have theirs.
fn text_room(ui: &Ui) -> f32 {
    let spacing = ui.spacing().item_spacing.y;
    let buttons = spacing.mul_add(2.0, ui.tokens().sizes.dialog_gap + PILL_HEIGHT);
    (ui.available_height() - buttons).max(MIN_TEXT_HEIGHT)
}
