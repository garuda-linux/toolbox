//! One app: its picture, its name with what waits for it, two lines of
//! its description, the menu and the switch.

use std::sync::Arc;

use egui::{Label, Ui};
use toolbox_core::Toggle;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    RowState, Tone, row_height, row_menu, split_row, status_dot, switch, tag, text_block_height,
};
use toolbox_plan::PackageName;

use super::app_row::{AppRow, AppRows, DESCRIPTION_LINES};
use crate::command::DiscoverCommand;
use crate::pages::PageContext;
use crate::pages::rows::{
    Initial, MENU_AND_SWITCH, actions, package_picture, picture_height, picture_slot,
};

/// A row's height in the fonts in use before it was measured: the
/// picture or a text of two description lines, whichever is taller.
pub fn height(ui: &Ui) -> f32 {
    row_height(
        ui,
        text_block_height(ui, DESCRIPTION_LINES).max(picture_height(ui, false)),
    )
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, row: &AppRow, state: &RowState) {
    split_row(
        ui,
        MENU_AND_SWITCH,
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

                package_picture(ui, context.pictures, row.package.as_ref(), initial, at);
            });
        },
        |ui| {
            row_menu(ui, state, &row.menu, |ui| {
                actions::menu::<AppRows>(ui, row, context.commands);
            });
            ui.add_space(ui.tokens().sizes.control_gap);

            // The website and the name are no change, so only the switch
            // rests while a plan runs or when the app is kept.
            let enabled = !context.busy && row.kept.is_none();
            let flipped = switch(ui, row.on, &row.title, enabled).clicked();
            if flipped || (state.toggle && enabled) {
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

        if let Some(category) = &row.category {
            tag(ui, category, Tone::Neutral);
        }

        let note = match (&row.queued, &row.kept) {
            (Some(queued), _) => Some((queued, Tone::Accent)),
            (None, Some(kept)) => Some((kept, Tone::Neutral)),
            (None, None) => None,
        };

        if let Some((note, tone)) = note {
            status_dot(ui, note, tone);
        }
    });

    ui.add(Label::new(Arc::clone(&row.description)));
}
