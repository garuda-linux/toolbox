//! The reports on offer, a row each in the shape of Maintenance's rows:
//! the report's name and what it holds, and Collect. The row of the
//! report that runs offers Cancel instead.
//!
//! Seven rows leave the report too little room in a default window, so
//! they fold while one shows.

use egui::Ui;
use toolbox_core::features::diagnostics::{Collector, Selection};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{
    ActionRow, ButtonKind, Tone, card, card_row, disclosure, pill_button, section_intro,
};
use toolbox_services::diagnostics::page::State;

use crate::command::DiagnosticsCommand;
use crate::diagnostics::Diagnostics;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;

/// In the order the card lists them, the full report last.
const CHOICES: [Selection; 7] = [
    Selection::One(Collector::Inxi),
    Selection::One(Collector::Health),
    Selection::One(Collector::SystemdAnalyze),
    Selection::One(Collector::Journal),
    Selection::One(Collector::PacmanLog),
    Selection::One(Collector::Dmesg),
    Selection::All,
];

/// While a report shows, the rows fold behind `open`, so the report keeps
/// the room of the window.
pub fn show(ui: &mut Ui, context: &PageContext<'_>, open: &mut bool) {
    let ctx = ui.ctx().clone();
    let page = context.service::<Diagnostics>().page();
    if page.state == State::Report && !page.report.is_empty() {
        let another = i18n::text(&ctx, "diagnostics-another");
        if disclosure(ui, &another, *open).clicked() {
            *open ^= true;
        }

        if !*open {
            return;
        }

        ui.add_space(ui.tokens().sizes.control_gap);
    } else {
        let title = i18n::text(&ctx, "diagnostics-reports");
        let intro = i18n::text(&ctx, "diagnostics-idle-detail");
        section_intro(ui, icons::OFFICE_REPORT, &title, &intro);
    }

    card(ui, |ui| {
        for (index, choice) in CHOICES.iter().enumerate() {
            card_row(ui, index == 0, |ui| row(ui, context, *choice));
        }
    });
}

fn row(ui: &mut Ui, context: &PageContext<'_>, choice: Selection) {
    let ctx = ui.ctx().clone();
    let page = context.service::<Diagnostics>().page();
    let name = super::name(&ctx, choice.as_str());
    let detail = i18n::text(
        &ctx,
        &format!("diagnostics-{}-description", choice.as_str()),
    );

    let running = page.state == State::Running && page.selection == choice.as_str();

    ActionRow::new(icon(choice), Tone::Neutral, &name)
        .detail(&detail)
        .show(ui, |ui| {
            if running {
                let cancel = i18n::text(&ctx, "diagnostics-cancel");
                if pill_button(ui, &cancel, ButtonKind::Soft).clicked() {
                    context.commands.send(DiagnosticsCommand::Cancel);
                }

                return;
            }

            // Each Collect names its report, so a screen reader tells the
            // rows' buttons apart.
            let collect = i18n::text(&ctx, "diagnostics-collect");
            let spoken = i18n::text_with(
                &ctx,
                "diagnostics-collect-name",
                &[("name", Arg::from(&name))],
            );

            let pressed = pill_button(ui, &collect, ButtonKind::Soft);

            let label = spoken.into_owned();
            ui.ctx()
                .accesskit_node_builder(pressed.id, |node| node.set_label(label));

            if pressed.clicked() {
                context.commands.send(DiagnosticsCommand::Collect(choice));
            }
        });
}

const fn icon(choice: Selection) -> Icon {
    match choice {
        Selection::One(Collector::Inxi) => icons::COMPUTER,
        Selection::One(Collector::Health) => icons::VIEW_TASK,
        Selection::One(Collector::SystemdAnalyze) => icons::SPEEDOMETER,
        Selection::One(Collector::Journal) => icons::TEXT_X_GENERIC,
        Selection::One(Collector::PacmanLog) => icons::PACKAGE,
        Selection::One(Collector::Dmesg) => icons::UTILITIES_TERMINAL,
        Selection::All => icons::OFFICE_REPORT,
    }
}
