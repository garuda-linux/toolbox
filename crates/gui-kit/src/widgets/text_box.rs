//! A box of several lines of text, filled as the one-line fields are but
//! with rounded corners, since round ends on a tall box would cut its
//! text.

use egui::{Frame, Margin, Response, ScrollArea, Shape, TextEdit, Ui};

use super::field::{PADDING_X, paint_field};
use crate::theme::RADIUS;

/// Room above and below the text, so its first line does not touch the
/// box's top.
const PADDING_Y: i8 = 10;

/// `edit` in a filled box as wide as the room, named `name`, that
/// scrolls both ways once its text is taller than `max_height`. The
/// caller keeps the text, font and whether it can be changed.
pub fn text_box(ui: &mut Ui, edit: TextEdit<'_>, name: &str, max_height: f32) -> Response {
    // The fill depends on the focus the text only reports once drawn.
    let fill_slot = ui.painter().add(Shape::Noop);
    let shown = Frame::NONE
        .inner_margin(Margin::symmetric(PADDING_X, PADDING_Y))
        .show(ui, |ui| {
            ScrollArea::both()
                .id_salt(("text_box", name))
                .max_height(max_height)
                .show(ui, |ui| {
                    ui.add(edit.frame(Frame::NONE).desired_width(f32::INFINITY))
                })
                .inner
        });

    let response = shown.inner;

    // The ring goes round the whole box, not the text that holds the
    // focus inside it.
    paint_field(
        ui,
        fill_slot,
        shown.response.rect,
        RADIUS,
        response.has_focus(),
    );

    let label = name.to_owned();
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(label));

    response
}
