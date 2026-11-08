//! One unit: its name with tags for its state, its description, the
//! boot switch and the menu of what runs at once or waits.

use std::sync::Arc;

use egui::{Label, Ui};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{
    RowState, Tone, line, rich, row_height, row_menu, split_row, status_dot, switch, tag,
    text_block_height,
};

use super::unit_row::{UnitRow, UnitRows};
use crate::command::ServiceCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::{MENU_AND_SWITCH, actions};

/// Every row's height in the fonts in use: the name and one line of
/// description.
pub fn height(ui: &Ui) -> f32 {
    row_height(ui, text_block_height(ui, 1))
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, row: &UnitRow, state: &RowState) {
    let caption = i18n::text(ui.ctx(), "services-at-boot");
    let secondary = ui.tokens().secondary;
    let caption_width = line(ui, &caption, Weight::Regular, SMALL, secondary)
        .size()
        .x;

    let caption_room = caption_width + ui.tokens().sizes.text_to_icon;

    split_row(
        ui,
        MENU_AND_SWITCH + caption_room,
        |ui| {
            toolbox_gui_kit::widgets::title_line(ui, false, |ui| {
                ui.add(Label::new(Arc::clone(&row.title)).truncate());
                status_dot(ui, &row.state.0, row.state.1);
                if let Some((file, hint)) = &row.file {
                    tag(ui, file, Tone::Neutral).on_hover_text(hint.as_ref());
                }

                for queued in &row.queued {
                    status_dot(ui, queued, Tone::Accent);
                }
            });

            ui.add(Label::new(Arc::clone(&row.description)).truncate());
        },
        |ui| {
            // The menu, the switch's caption, then the switch at the edge,
            // the order of the App Catalog's rows and of Tab.
            let busy = i18n::text(ui.ctx(), "services-why-busy");
            ui.add_enabled_ui(!context.busy, |ui| {
                row_menu(ui, state, &row.menu, |ui| {
                    actions::menu::<UnitRows>(ui, row, context.commands);
                })
            })
            .inner
            .on_disabled_hover_text(busy);

            ui.add_space(ui.tokens().sizes.control_gap);
            ui.add(
                Label::new(rich(ui, &caption, Weight::Regular, SMALL, secondary)).selectable(false),
            );
            ui.add_space(ui.tokens().sizes.text_to_icon - ui.spacing().item_spacing.x);

            let enabled = row.switchable && !context.busy;
            let name = i18n::text_with(
                ui.ctx(),
                "services-start-at-boot",
                &[("unit", row.unit.as_str())],
            );

            let flipped = switch(ui, row.enabled, &name, enabled).clicked();
            if flipped || (state.toggle && enabled) {
                context.commands.send(ServiceCommand::Enable {
                    scope: row.scope,
                    unit: row.unit.clone(),
                    on: !row.enabled,
                });
            }
        },
    );
}
