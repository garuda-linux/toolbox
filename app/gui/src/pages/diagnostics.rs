//! The Diagnostics page: pick a report, collect it, read it with the
//! programs' colours, then copy it or share it as an encrypted paste.
//!
//! Collecting changes nothing on the system, so it does not wait for a
//! running plan. Copy and upload always take the plain text.

mod report;
#[cfg(test)]
mod tests;

use egui::{Label, Ui};
use toolbox_core::features::diagnostics::{Collector, Selection};
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, Select, Tone, banner, card, empty_state, loading_label, pill_button, rich,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_system::diagnostics::page::State;

use super::header::title_header;
use super::{PageContext, PageView};
use crate::command::DiagnosticsCommand;
use crate::diagnostics::Diagnostics;
use crate::i18n;
use crate::share::view::{self as share, Question, ShareView, Subject, padded_card};

const CHOICES: [Selection; 7] = [
    Selection::One(Collector::Inxi),
    Selection::One(Collector::Health),
    Selection::One(Collector::SystemdAnalyze),
    Selection::One(Collector::Journal),
    Selection::One(Collector::PacmanLog),
    Selection::One(Collector::Dmesg),
    Selection::All,
];

const NARROW_CONTROL: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_narrow;

#[derive(Debug)]
pub struct DiagnosticsPage {
    picked: Selection,
    asking: Option<Question>,
}

impl Default for DiagnosticsPage {
    fn default() -> Self {
        Self {
            picked: Selection::One(Collector::Inxi),
            asking: None,
        }
    }
}

impl PageView for DiagnosticsPage {
    fn header(&mut self, ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        title_header(ui, "page-diagnostics", "page-diagnostics-subtitle");
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("diagnostics ui").entered();
        self.picker(ui, context);
        ui.add_space(ui.tokens().sizes.section_gap);

        let page = context.service::<Diagnostics>().page();
        let view = ShareView {
            upload: &page.share,
            subject: Subject::Report,
            commands: context.commands,
            hide: DiagnosticsCommand::HideShare.into(),
        };

        if share::status(ui, &view, &mut self.asking) {
            ui.add_space(ui.tokens().sizes.section_gap);
        }

        if page.state == State::Report && !page.report.is_empty() {
            report::show(ui, context);
        } else {
            status(ui, context);
        }

        if let Some(shareable) = &page.shareable
            && share::ask(ui.ctx(), &view, shareable, &mut self.asking)
        {
            context.commands.send(DiagnosticsCommand::Upload);
        }
    }
}

impl DiagnosticsPage {
    fn picker(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        let page = context.service::<Diagnostics>().page();
        let names: Vec<String> = CHOICES
            .iter()
            .map(|choice| name(&ctx, choice.as_str()))
            .collect();

        let shown: Vec<&str> = names.iter().map(String::as_str).collect();
        let current = CHOICES.iter().position(|choice| *choice == self.picked);
        padded_card(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let label = i18n::text(&ctx, "diagnostics-report-picker");
                let picked = Select::new(&label, &shown)
                    .selected(current)
                    .width(NARROW_CONTROL)
                    .show(ui);

                if let Some(choice) = picked.and_then(|index| CHOICES.get(index)) {
                    self.picked = *choice;
                }

                let collect = i18n::text(&ctx, "diagnostics-collect");

                // Once a link exists, copying it is what the page leads to.
                let kind = if page.share.paste().is_some() {
                    ButtonKind::Soft
                } else {
                    ButtonKind::Primary
                };

                if pill_button(ui, &collect, kind).clicked() {
                    context
                        .commands
                        .send(DiagnosticsCommand::Collect(self.picked));
                }

                if page.state == State::Running {
                    let cancel = i18n::text(&ctx, "diagnostics-cancel");
                    if pill_button(ui, &cancel, ButtonKind::Soft).clicked() {
                        context.commands.send(DiagnosticsCommand::Cancel);
                    }
                }

                if page.state == State::Report && !page.report.is_empty() {
                    self.report_buttons(ui, context);
                }
            });

            let id = format!("diagnostics-{}-description", self.picked.as_str());
            let secondary = ui.tokens().secondary;
            let description = i18n::text(&ctx, &id);
            ui.add_space(ui.tokens().sizes.control_gap);
            ui.add(Label::new(rich(ui, &description, Weight::Regular, BODY, secondary)).wrap());
        });
    }

    fn report_buttons(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        let page = context.service::<Diagnostics>().page();
        if pill_button(ui, &i18n::text(&ctx, "diagnostics-copy"), ButtonKind::Soft).clicked() {
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

fn status(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let page = context.service::<Diagnostics>().page();
    let selection = name(&ctx, &page.selection);
    let named = |id: &str| i18n::text_with(&ctx, id, &[("name", &selection)]).into_owned();

    match page.state {
        State::Idle => {
            empty_state(
                ui,
                icons::OFFICE_REPORT,
                &i18n::text(&ctx, "diagnostics-idle"),
                &i18n::text(&ctx, "diagnostics-idle-detail"),
            );
        }
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
