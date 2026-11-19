//! The merged text, edited by hand, for what no pick of whole changes
//! gives. A text equal to either file queues that file.

use egui::{TextEdit, Ui};
use toolbox_core::features::merges::{choice, edited};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{ButtonKind, Tone, banner, pill_button, text_box};
use toolbox_plan::ConfigText;
use toolbox_system::MergePair;

use super::MergeDialog;
use crate::command::HomeCommand;
use crate::i18n;
use crate::pages::PageContext;

/// The buttons' row below the text.
const FOOTER: f32 = 64.0;

pub(super) fn show(
    ui: &mut Ui,
    dialog: &mut MergeDialog,
    context: &PageContext<'_>,
    pair: &MergePair,
) {
    let ctx = ui.ctx().clone();
    let (Ok(copied), Ok(current)) = (&pair.copied, &pair.current) else {
        dialog.editing = None;
        return;
    };

    let Some((_, text)) = dialog.editing.as_mut() else {
        return;
    };

    let name = i18n::text(&ctx, "home-merges-edit-name");
    let height = (ui.available_height() - FOOTER).max(FOOTER);
    let edit = TextEdit::multiline(text)
        .code_editor()
        .desired_width(f32::INFINITY);

    text_box(
        ui,
        ("toolbox_gui::home::merges::edit", &*name),
        edit,
        height,
    );
    ui.add_space(ui.tokens().sizes.control_gap);

    let current = current.as_ref().map(ConfigText::as_str);
    let resolution = edited(text, current.unwrap_or_default(), copied.as_str());

    if resolution.is_err() {
        banner(
            ui,
            Tone::Error,
            &i18n::text(&ctx, "home-merges-edit-invalid"),
            &[],
        );
    }

    let mut back = false;
    ui.horizontal(|ui| {
        back = pill_button(
            ui,
            &i18n::text(&ctx, "home-merges-edit-back"),
            ButtonKind::Ghost,
        )
        .clicked();

        let queued = ui
            .enabled_ui(resolution.is_ok(), |ui| {
                pill_button(
                    ui,
                    &i18n::text(&ctx, "home-merges-edit-queue"),
                    ButtonKind::Primary,
                )
            })
            .inner
            .clicked();

        if queued && let Ok(resolution) = resolution {
            context.commands.send(HomeCommand::Merge {
                copy: pair.copy.clone(),
                choice: Some(choice(resolution, current, copied.as_str())),
            });
            back = true;
        }
    });

    if back {
        dialog.editing = None;
    }
}
