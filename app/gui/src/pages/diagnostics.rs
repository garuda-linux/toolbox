//! The Diagnostics page: a row per report to collect, then the report with
//! the programs' colours under its header line, to copy or to share as an
//! encrypted paste.
//!
//! Collecting changes nothing on the system, so it does not wait for a
//! running plan. Copy and upload always take the plain text.

mod report;
mod rows;
#[cfg(test)]
mod tests;

use egui::{Label, Ui};
use toolbox_core::features::diagnostics::Collector;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::widgets::{
    BodyShape, ButtonKind, Tone, banner, card, loading_label, pill_button, rich,
};
use toolbox_services::diagnostics::page::State;

use super::header::title_header;
use super::{PageContext, PageView};
use crate::command::DiagnosticsCommand;
use crate::diagnostics::Diagnostics;
use crate::i18n;
use crate::share::view::{self as share, Question, ShareView, Subject};

#[derive(Debug, Default)]
pub struct DiagnosticsPage {
    asking: Option<Question>,
    /// The report rows are unfolded while a report shows.
    rows_open: bool,
    /// Long lines wrap at the viewer's edge instead of scrolling sideways.
    wrap: bool,
    query: String,
    shown: report::Shown,
}

impl PageView for DiagnosticsPage {
    fn header(&mut self, ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        title_header(ui, "page-diagnostics", "page-diagnostics-subtitle");
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("diagnostics ui").entered();
        rows::show(ui, context, &mut self.rows_open);
        ui.add_space(ui.tokens().sizes.card_gap);

        let page = context.service::<Diagnostics>().page();
        let view = ShareView {
            upload: &page.share,
            subject: Subject::Report,
            commands: context.commands,
            hide: DiagnosticsCommand::HideShare.into(),
        };

        if share::row(ui, &view, &mut self.asking) {
            ui.add_space(ui.tokens().sizes.card_gap);
        }

        if page.state == State::Report && !page.report.is_empty() {
            self.report(ui, context);
        } else {
            status(ui, context);
        }

        if let Some(shareable) = &page.shareable
            && share::ask(ui.ctx(), &view, shareable, &mut self.asking)
        {
            context.commands.send(DiagnosticsCommand::Upload);
        }
    }

    /// The report box fills the window below the rows.
    fn shape(&self) -> BodyShape {
        BodyShape::Filled
    }
}

impl DiagnosticsPage {
    fn report_buttons(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        let page = context.service::<Diagnostics>().page();

        // Once a link exists, copying it is what the page leads to.
        let kind = if page.share.paste().is_some() {
            ButtonKind::Soft
        } else {
            ButtonKind::Primary
        };

        if pill_button(ui, &i18n::text(&ctx, "diagnostics-copy"), kind).clicked() {
            share::copy(
                &ctx,
                context.commands,
                &page.report,
                "diagnostics-report-copied",
            );
        }

        let share = i18n::text(&ctx, "diagnostics-share");

        // The redacted report follows the report in a moment.
        let ready = page.shareable.is_some() && !page.share.running();
        let shared = ui
            .add_enabled_ui(ready, |ui| pill_button(ui, &share, ButtonKind::Soft))
            .inner
            .on_hover_text(i18n::text(&ctx, "diagnostics-share-tooltip"));

        if shared.clicked() {
            self.asking = Some(Question::Upload);
        }
    }
}

fn running(ui: &mut Ui, text: &str, detail: &str) {
    let tokens = ui.tokens();
    card(ui, |ui| {
        egui::Frame::new()
            .inner_margin(tokens.sizes.card_padding)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = tokens.sizes.tight_gap;
                loading_label(ui, text);
                let words = rich(ui, detail, Weight::Regular, BODY, tokens.secondary);
                ui.add(Label::new(words).wrap());
            });
    });
}

/// Nothing before the first collection: the rows above say what to do.
fn status(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let page = context.service::<Diagnostics>().page();
    let selection = name(&ctx, &page.selection);
    let named = |id: &str| i18n::text_with(&ctx, id, &[("name", &selection)]).into_owned();

    match page.state {
        State::Idle => {}
        State::Running => {
            let text = if page.part.is_empty() {
                named("diagnostics-collecting")
            } else {
                let part = name(&ctx, &page.part);
                let (step, count) = (page.step.to_string(), Collector::ALL.len().to_string());
                i18n::text_with(
                    &ctx,
                    "diagnostics-collecting-step",
                    &[("name", &part), ("step", &step), ("count", &count)],
                )
                .into_owned()
            };

            running(
                ui,
                &text,
                &i18n::text(&ctx, "diagnostics-collecting-detail"),
            );
        }
        State::Report => {
            banner(ui, Tone::Neutral, &named("diagnostics-empty"), &[]);
        }
        State::Unavailable => {
            let text = format!(
                "{} {}",
                named("diagnostics-unavailable"),
                i18n::text(&ctx, "diagnostics-unavailable-detail")
            );

            banner(ui, Tone::Warning, &text, &[]);
        }
        State::Refused(refusal) => {
            let text = named(&format!("diagnostics-{}", refusal.code()));
            banner(ui, Tone::Warning, &text, &[]);
        }
        State::Failed => {
            let text = i18n::text_with(
                &ctx,
                "diagnostics-failed",
                &[("name", &selection), ("reason", &page.problem)],
            );

            banner(ui, Tone::Error, &text, &[]);
        }
    }
}

/// Empty before the first collection, which has no report to name.
fn name(ctx: &egui::Context, id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }

    i18n::text(ctx, &format!("diagnostics-{id}")).into_owned()
}
