//! One app: its picture, its name with what waits for it, two lines of
//! its description, the menu and Install, Remove or Take back. Beside the
//! details the row keeps its picture, its name and one line of description.

use std::sync::Arc;

use egui::text::LayoutJob;
use egui::{Key, Label, Modifiers, Ui};
use toolbox_core::Toggle;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    RowState, row_height, row_menu, split_row_in, status_dot, text_block_height,
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

/// A row beside the details: its name and one line of its description.
pub(super) fn compact_height(ui: &Ui) -> f32 {
    row_height(ui, text_block_height(ui, 1).max(picture_height(ui, false)))
}

/// Draws one app and returns whether it asks for its details.
///
/// Full width, a click on the row or Enter on it opens the details, and
/// Space presses the row's button. Beside the details the row has no
/// button, so Space, a click, Enter and the arrow keys that reach the row
/// all open it.
pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    row: &AppRow,
    state: &RowState,
    beside: bool,
) -> bool {
    let enter = state.response.has_focus()
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter));

    if beside {
        picture_and_text(ui, context, row, &row.summary);
        return state.toggle || state.keyed || enter;
    }

    split_row_in(
        ui,
        state.row,
        control::width(ui, row.control, Setting::Button),
        |ui| picture_and_text(ui, context, row, &row.description),
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
            if control::show(ui, button(context, row), state) {
                send_toggle(context, row);
            }
        },
    );

    state.response.clicked() || enter
}

/// The app's install or remove button, as the row and the details draw it.
pub fn button<'row>(context: &PageContext<'_>, row: &'row AppRow) -> control::Shown<'row> {
    control::Shown {
        control: row.control,
        on: row.on,
        title: &row.title,
        enabled: !context.busy && row.kept.is_none(),
        unknown: false,
        setting: Setting::Button,
    }
}

pub(super) fn send_toggle(context: &PageContext<'_>, row: &AppRow) {
    context.commands.send(DiscoverCommand::Toggle {
        app: row.id.clone(),
        wish: if row.on { Toggle::Off } else { Toggle::On },
    });
}

fn picture_and_text(
    ui: &mut Ui,
    context: &PageContext<'_>,
    row: &AppRow,
    description: &Arc<LayoutJob>,
) {
    ui.horizontal_top(|ui| {
        let at = picture_slot(ui, false);
        ui.vertical(|ui| text(ui, row, description));
        picture(ui, context, row, at);
    });
}

/// The app's picture in `at`, or its letter tile. An app without a package
/// keeps a hue by its catalog id, which no language changes.
pub fn picture(ui: &Ui, context: &PageContext<'_>, row: &AppRow, at: egui::Rect) {
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
}

fn text(ui: &mut Ui, row: &AppRow, description: &Arc<LayoutJob>) {
    toolbox_gui_kit::widgets::title_line(ui, false, |ui| {
        ui.add(Label::new(Arc::clone(&row.title_job)).truncate());

        if let Some((note, tone)) = row.note() {
            status_dot(ui, note, tone);
        }
    });

    ui.add(Label::new(Arc::clone(description)));
}
