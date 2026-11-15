//! One app: its picture, its name with what waits for it, two lines of
//! its description, the menu and Install, Remove or Take back.

use std::sync::Arc;

use egui::{Label, Ui};
use toolbox_core::Toggle;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    RowState, Tone, row_height, row_menu, split_row_in, status_dot, text_block_height,
};
use toolbox_plan::PackageName;

use super::app_row::{AppRow, AppRows};
use crate::artwork::Artwork;
use crate::command::DiscoverCommand;
use crate::pages::PageContext;
use crate::pages::rows::{Initial, actions, package_picture, picture_height, picture_slot};
use crate::pages::toggles::control::{self, DESCRIPTION_LINES, Setting};

/// A row's height in the fonts in use before it was measured: the
/// picture or a text of two description lines, whichever is taller.
pub fn height(ui: &Ui) -> f32 {
    row_height(
        ui,
        text_block_height(ui, DESCRIPTION_LINES).max(picture_height(ui, false)),
    )
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, row: &AppRow, state: &RowState) {
    split_row_in(
        ui,
        state.row,
        control::width(ui, row.control, Setting::Button),
        |ui| {
            ui.horizontal_top(|ui| {
                let at = picture_slot(ui, false);
                ui.vertical(|ui| text(ui, row));

                // An app without a package keeps a hue by its catalog id,
                // which no language changes.
                let initial = Initial {
                    shown: &row.title,
                    package: row.package.as_ref().map_or(&row.id, PackageName::as_str),
                };

                package_picture(
                    ui,
                    context.service::<Artwork>().pictures(),
                    row.package.as_ref(),
                    initial,
                    at,
                );
            });
        },
        |ui| {
            // `control::width` reserves exactly menu, gap and control, so no
            // item spacing on top, or the row overflows its slot.
            ui.spacing_mut().item_spacing.x = 0.0;
            row_menu(ui, state, &row.menu, |ui| {
                actions::menu::<AppRows>(ui, row, context.commands);
            });
            ui.add_space(ui.tokens().sizes.control_gap);

            // The website and the name are no change, so only the control
            // rests while a plan runs or when the app is kept.
            let shown = control::Shown {
                control: row.control,
                on: row.on,
                title: &row.title,
                enabled: !context.busy && row.kept.is_none(),
                unknown: false,
                setting: Setting::Button,
            };

            if control::show(ui, shown, state) {
                context.commands.send(DiscoverCommand::Toggle {
                    app: row.id.clone(),
                    wish: if row.on { Toggle::Off } else { Toggle::On },
                });
            }
        },
    );
}

fn text(ui: &mut Ui, row: &AppRow) {
    toolbox_gui_kit::widgets::title_line(ui, false, |ui| {
        ui.add(Label::new(Arc::clone(&row.title_job)).truncate());

        let note = match (&row.queued, &row.kept, &row.installed) {
            (Some(queued), _, _) => Some((queued, Tone::Accent)),
            (None, Some(kept), _) => Some((kept, Tone::Neutral)),
            (None, None, Some(installed)) => Some((installed, Tone::Success)),
            (None, None, None) => None,
        };

        if let Some((note, tone)) = note {
            status_dot(ui, note, tone);
        }
    });

    ui.add(Label::new(Arc::clone(&row.description)));
}
