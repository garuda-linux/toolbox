//! The text of a file that is only shown: selectable, never changed.
//! Closing the dialog drops the text.

use egui::{Context, Id, Label, TextEdit, Ui};
use toolbox_core::features::configs::viewing::{Phase, Viewing};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    ButtonKind, DialogWidth, ModalWrapper, Tone, banner, loading_label, pill_button, pill_width,
    text_box_filling,
};

use super::texts;
use crate::command::ConfigsCommand;
use crate::i18n;
use crate::pages::PageContext;

pub fn show(ctx: &Context, context: &PageContext<'_>, viewing: &Viewing) {
    let path = viewing.path().map(ToString::to_string).unwrap_or_default();
    let title = i18n::text_with(ctx, "configs-view-title", &[("path", &path)]);
    let close = i18n::text(ctx, "dialog-close");
    let outcome = ModalWrapper::new(Id::unique("toolbox_gui::configs::view"), &title)
        .title_bar(&close)
        .width(DialogWidth::Wide)
        .full_height()
        .show(ctx, |ui| {
            super::filled(
                ui,
                |ui| body(ui, viewing, &path),
                |ui| buttons(ui, context, viewing),
            )
        });

    if outcome.closed || outcome.inner {
        context.commands.send(ConfigsCommand::CloseView);
    }
}

fn body(ui: &mut Ui, viewing: &Viewing, path: &str) {
    let ctx = ui.ctx().clone();
    match viewing.phase() {
        Phase::Idle => {}
        Phase::Reading => {
            let text = i18n::text_with(&ctx, "configs-status-reading", &[("path", path)]);
            loading_label(ui, &text);
        }
        Phase::Failed => {
            let text = texts::view_problem(&ctx, viewing.problem().code, path);
            banner(ui, Tone::Error, &text, &[]);
        }
        Phase::Shown => shown(ui, viewing, path),
    }
}

fn shown(ui: &mut Ui, viewing: &Viewing, path: &str) {
    let ctx = ui.ctx().clone();
    let Some(read) = viewing.viewed() else {
        return;
    };

    let target = read.path.to_string();
    if target != path {
        let text = i18n::text_with(
            &ctx,
            "configs-view-link",
            &[("path", path), ("target", &target)],
        );

        ui.add(Label::new(text).wrap());
    }

    let note = texts::problem(&ctx, "read-only", path);
    ui.add(Label::new(note).wrap());
    ui.add_space(ui.tokens().sizes.card_gap);

    let mut text = read.content.as_str();
    let name = i18n::text_with(&ctx, "configs-view-title", &[("path", path)]).into_owned();
    text_box_filling(ui, TextEdit::multiline(&mut text).code_editor(), &name);
}

/// Copy all and Close. Returns whether Close was clicked.
fn buttons(ui: &mut Ui, context: &PageContext<'_>, viewing: &Viewing) -> bool {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let labels = [text("configs-copy-all"), text("configs-close")];
    ui.horizontal(|ui| {
        let gap = ui.tokens().sizes.control_gap;
        ui.spacing_mut().item_spacing.x = gap;
        let width = labels
            .iter()
            .map(|label| pill_width(ui, label))
            .sum::<f32>()
            + gap;

        ui.add_space((ui.available_width() - width).max(0.0));
        if let Some(read) = viewing.viewed()
            && pill_button(ui, &text("configs-copy-all"), ButtonKind::Soft).clicked()
        {
            crate::share::view::copy(
                &ctx,
                context.commands,
                read.content.as_str(),
                "configs-copied",
            );
        }

        pill_button(ui, &text("configs-close"), ButtonKind::Primary).clicked()
    })
    .inner
}
