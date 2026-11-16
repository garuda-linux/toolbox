//! One unit: its name with tags for its state, its description, the
//! boot switch and the menu of what runs at once or waits. One caption
//! above the list names the switches' column.

use std::sync::Arc;

use egui::{Label, Rect, Sense, Ui, pos2, vec2};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{
    RowState, Tone, line, rich, row_height, row_menu, split_row_in, status_dot, status_word, tag,
    text_block_height,
};

use super::unit_row::{UnitRow, UnitRows};
use crate::command::ServiceCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::{MENU_AND_SWITCH, SWITCH_WIDTH, actions, row_switch};

/// Every row's height in the fonts in use: the name and one line of
/// description.
pub fn height(ui: &Ui) -> f32 {
    row_height(ui, text_block_height(ui, 1))
}

/// Whether screen readers hear the unit's state change: on the row the
/// keyboard is on, and on a failure wherever it shows. Every row live
/// would read the whole list at each relist.
pub fn announced(state: &RowState, tone: Tone) -> bool {
    tone == Tone::Error || state.response.has_focus()
}

/// "At boot" centred over the switches at the rows' right edge.
pub fn column_caption(ui: &mut Ui) {
    let caption = i18n::text(ui.ctx(), "services-at-boot");
    let secondary = ui.tokens().secondary;
    let size = line(ui, &caption, Weight::Regular, SMALL, secondary).size();
    let inset = ui.tokens().sizes.row_padding.x;
    let (room, _) = ui.allocate_exact_size(vec2(ui.available_width(), size.y), Sense::hover());
    let middle = room.right() - inset - SWITCH_WIDTH / 2.0;
    let at = Rect::from_min_size(pos2(middle - size.x / 2.0, room.top()), size);

    ui.put(
        at,
        Label::new(rich(ui, &caption, Weight::Regular, SMALL, secondary)).selectable(false),
    );
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, row: &UnitRow, state: &RowState) {
    let live = announced(state, row.state.1);

    split_row_in(
        ui,
        state.row,
        MENU_AND_SWITCH,
        |ui| {
            toolbox_gui_kit::widgets::title_line(ui, false, |ui| {
                ui.add(Label::new(Arc::clone(&row.title)).truncate());
                if live {
                    status_word(ui, &row.state.0, row.state.1);
                } else {
                    status_dot(ui, &row.state.0, row.state.1);
                }

                if let Some((file, hint)) = &row.file {
                    // The focused row shows it too, so the keyboard reaches
                    // what the pointer does.
                    let shown = tag(ui, file, Tone::Neutral).on_hover_text(hint.as_ref());
                    if state.response.has_focus() {
                        shown.show_tooltip_text(hint.as_ref());
                    }
                }

                for queued in &row.queued {
                    status_dot(ui, queued, Tone::Accent);
                }
            });

            ui.add(Label::new(Arc::clone(&row.description)).truncate());
        },
        |ui| {
            // Menu before switch, as in the App Catalog and in Tab order.
            let busy = i18n::text(ui.ctx(), "services-why-busy");
            ui.enabled_ui(!context.busy, |ui| {
                row_menu(ui, state, &row.menu, |ui| {
                    actions::menu::<UnitRows>(ui, row, context.commands);
                })
            })
            .inner
            .on_disabled_hover_text(busy);

            ui.add_space(ui.tokens().sizes.control_gap);

            let name = i18n::text_with(
                ui.ctx(),
                "services-start-at-boot",
                &[("unit", row.unit.as_str())],
            );

            // The list waits in a disabled Ui while a plan runs.
            if row_switch(ui, state, row.enabled, &name, row.switchable) {
                context.commands.send(ServiceCommand::Enable {
                    scope: row.scope,
                    unit: row.unit.clone(),
                    on: !row.enabled,
                });
            }
        },
    );
}
