//! Overwritten files are kept as backups beside the originals.

use egui::{Align, Id, Label, Layout, Ui};
use toolbox_gui_kit::widgets::{
    ButtonKind, CheckRowTexts, Confirmation, ModalWrapper, Tone, banner, card, card_row,
    check_row_with_line, confirm_buttons, empty_state, loading_label, pill_button, pill_width,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_system::skel::shelf::SkelRow;

use super::heading;
use crate::command::MaintenanceCommand;
use crate::i18n;
use crate::maintenance::Maintenance;
use crate::pages::PageContext;

pub fn show(ui: &mut Ui, context: &PageContext<'_>, confirming: &mut bool) {
    let ctx = ui.ctx().clone();
    let shelf = context.service::<Maintenance>().shelf();
    heading(ui, icons::DOCUMENT_REVERT, "maintenance-reset");
    if shelf.rows().is_empty() {
        empty_state(
            ui,
            icons::HELP_ABOUT,
            &i18n::text(&ctx, "maintenance-reset-empty"),
            &i18n::text(&ctx, "maintenance-reset-empty-detail"),
        );
        return;
    }

    outcome(ui, context);
    let resetting = shelf.resetting();
    card(ui, |ui| {
        for (index, entry) in shelf.rows().iter().enumerate() {
            card_row(ui, index == 0, |ui| {
                entry_row(ui, context, entry, resetting);
            });
        }
    });

    ui.add_space(ui.tokens().sizes.card_gap);
    if resetting {
        loading_label(ui, &i18n::text(&ctx, "maintenance-reset-busy"));
    } else {
        let text = i18n::text(&ctx, "maintenance-reset-button");
        let width = pill_width(ui, &text);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            Layout::right_to_left(Align::Center),
            |ui| {
                ui.set_min_width(width);
                let chosen = shelf.chosen_count() > 0;
                let button =
                    ui.add_enabled_ui(chosen, |ui| pill_button(ui, &text, ButtonKind::Primary));

                if button.inner.clicked() {
                    *confirming = true;
                }
            },
        );
    }

    if *confirming {
        ask(&ctx, context, confirming);
    }
}

fn entry_row(ui: &mut Ui, context: &PageContext<'_>, entry: &SkelRow, resetting: bool) {
    let on = context
        .service::<Maintenance>()
        .shelf()
        .is_chosen(&entry.entry);

    let files = entry.targets.join("\n");
    let line = i18n::text_with(
        ui.ctx(),
        "maintenance-reset-replaces",
        &[("files", &entry.targets.join(", "))],
    );

    let texts = CheckRowTexts {
        label: &entry.name,
        line: &line,
        full: &files,
    };

    if check_row_with_line(ui, on, texts, !resetting).clicked() {
        context.commands.send(MaintenanceCommand::Choose {
            entry: entry.entry.clone(),
            on: !on,
        });
    }
}

fn outcome(ui: &mut Ui, context: &PageContext<'_>) {
    let shelf = context.service::<Maintenance>().shelf();
    let outcome = shelf.outcome();
    if !outcome.done || shelf.resetting() {
        return;
    }

    let ctx = ui.ctx().clone();
    if outcome.failed.is_empty() {
        let text = i18n::text(&ctx, "maintenance-reset-done");
        banner(ui, Tone::Success, &text, &[]);
    } else {
        let names = outcome.failed.join(", ");
        let text = i18n::text_with(&ctx, "maintenance-reset-failed", &[("names", &names)]);
        banner(ui, Tone::Warning, &text, &[]);
    }

    ui.add_space(ui.tokens().sizes.card_gap);
}

fn ask(ctx: &egui::Context, context: &PageContext<'_>, confirming: &mut bool) {
    let shelf = context.service::<Maintenance>().shelf();
    let count = shelf.chosen_count().to_string();
    let title = i18n::text_with(ctx, "maintenance-reset-question", &[("count", &count)]);
    let close = i18n::text(ctx, "dialog-close");
    let outcome = ModalWrapper::new(Id::unique("toolbox_gui::maintenance::reset"), &title)
        .title_bar(&close)
        .show(ctx, |ui| {
            ui.add(Label::new(i18n::text(ctx, "maintenance-reset-files")).wrap());
            ui.add_space(ui.tokens().sizes.card_gap);
            for target in shelf.chosen_targets() {
                ui.add(Label::new(egui::RichText::new(target).monospace()).wrap());
            }

            confirm_buttons(
                ui,
                &i18n::text(ctx, "maintenance-reset-cancel"),
                &i18n::text(ctx, "maintenance-reset-confirm"),
                ButtonKind::Danger,
            )
        });

    if outcome.inner == Some(Confirmation::Confirmed) {
        context.commands.send(MaintenanceCommand::Reset);
    }

    if outcome.closed || outcome.inner.is_some() {
        *confirming = false;
    }
}
