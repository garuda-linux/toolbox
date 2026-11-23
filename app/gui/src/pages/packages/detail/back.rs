use egui::Ui;
use toolbox_gui_kit::widgets::{
    ButtonKind, MIN_TEXT_WIDTH, icon_button, icon_button_width, pill_button_with_icon,
    pill_width_with_icon,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::super::PackagesPage;
use super::title::title_picture_width;
use crate::i18n;
use crate::i18n::Arg;

/// The details' back button, as Kirigami's: a flat button with the
/// previous page's name, or the icon alone where the title needs the room.
pub(super) struct Back {
    text: String,
    name: String,
    as_text: bool,
    pub(super) width: f32,
}

impl Back {
    pub(super) fn new(page: &PackagesPage, ui: &Ui, trailing: f32) -> Self {
        let ctx = ui.ctx().clone();
        let (text, name) = page.trail.last().map_or_else(
            || {
                (
                    i18n::text(&ctx, "packages-back").into_owned(),
                    i18n::text(&ctx, "packages-back-name").into_owned(),
                )
            },
            |(previous, _)| {
                let to = [("package", Arg::from(previous.as_str()))];
                (
                    i18n::text_with(&ctx, "packages-back-to", &to).into_owned(),
                    i18n::text_with(&ctx, "packages-back-to-name", &to).into_owned(),
                )
            },
        );

        // The title's room as `PageHeader` leaves it: the picture leads the
        // title too, and a gap follows the button, the picture and the title.
        let gap = ui.tokens().sizes.text_to_control;
        let text_width = pill_width_with_icon(ui, &text);
        let taken = text_width + title_picture_width(ui) + trailing;
        let room = (-gap).mul_add(3.0, ui.available_width() - taken);
        let as_text = room >= MIN_TEXT_WIDTH;
        let width = if as_text {
            text_width
        } else {
            icon_button_width()
        };

        Self {
            text,
            name,
            as_text,
            width,
        }
    }

    /// Whether it was pressed.
    pub(super) fn show(&self, ui: &mut Ui) -> bool {
        let icon = icons::GO_PREVIOUS;
        if self.as_text {
            pill_button_with_icon(ui, &self.text, ButtonKind::Ghost, icon)
                .accessible_name(&self.name)
                .clicked()
        } else {
            icon_button(ui, icon, &self.name).clicked()
        }
    }
}
