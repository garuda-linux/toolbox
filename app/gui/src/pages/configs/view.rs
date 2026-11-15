//! The text of a file that is only shown: selectable, never changed.
//! Closing the dialog drops the text.

use egui::{Context, Id, Label, TextEdit, Ui};
use toolbox_core::features::configs::viewing::{Phase, Viewing};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    ButtonKind, DialogWidth, ModalWrapper, Tone, banner, loading_label, pill_button, right_buttons,
    text_box,
};

use super::texts;
use crate::command::ConfigsCommand;
use crate::i18n;
use crate::pages::PageContext;

/// The frame `text_box` draws above and below its scrolled room.
const BOX_PADDING: f32 = 20.0;

pub fn show(ctx: &Context, context: &PageContext<'_>, viewing: &Viewing) {
    let path = viewing.path().map(ToString::to_string).unwrap_or_default();
    let title = i18n::text_with(ctx, "configs-view-title", &[("path", &path)]);
    let close = i18n::text(ctx, "dialog-close");

    // Only a read text needs the width, a line saying it is read or why
    // it was not fits a small dialog.
    let width = match viewing.phase() {
        Phase::Shown => DialogWidth::Wide,
        Phase::Idle | Phase::Reading | Phase::Failed => DialogWidth::Small,
    };

    let outcome = ModalWrapper::new(Id::unique("toolbox_gui::configs::view"), &title)
        .title_bar(&close)
        .width(width)
        .show(ctx, |ui| {
            super::stacked(
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
            if let Some(problem) = viewing.problem() {
                let text = texts::view_problem(&ctx, problem.code, path);
                banner(ui, Tone::Error, &text, &[]);
            }
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

    let note = i18n::text_with(&ctx, "configs-problem-read-only", &[("path", path)]);
    ui.add(Label::new(note).wrap());
    ui.add_space(ui.tokens().sizes.card_gap);

    let mut text = read.content.as_str();
    let name = i18n::text_with(&ctx, "configs-view-title", &[("path", path)]).into_owned();
    let room = (super::text_room(ui) - BOX_PADDING).max(super::MIN_TEXT_HEIGHT);
    text_box(
        ui,
        (&name, &name),
        TextEdit::multiline(&mut text).code_editor(),
        room,
    );
}

/// Copy all, once a text was read, and Close. Returns whether Close was
/// clicked.
fn buttons(ui: &mut Ui, context: &PageContext<'_>, viewing: &Viewing) -> bool {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let read = viewing.viewed();
    let copy_all = read.map(|_| text("configs-copy-all"));
    let close = text("configs-close");
    let labels: Vec<&str> = copy_all
        .iter()
        .map(String::as_str)
        .chain([close.as_str()])
        .collect();

    right_buttons(ui, &labels, |ui| {
        if let (Some(read), Some(copy_all)) = (read, &copy_all)
            && pill_button(ui, copy_all, ButtonKind::Soft).clicked()
        {
            crate::share::view::copy(
                &ctx,
                context.commands,
                read.content.as_str(),
                "configs-copied",
            );
        }

        pill_button(ui, &close, ButtonKind::Primary).clicked()
    })
}
