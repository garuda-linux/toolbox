//! A box of several lines of text, filled as the one-line fields are but
//! with rounded corners, since round ends on a tall box would cut its
//! text.

use egui::text::CCursor;
use egui::{Align, Frame, Margin, Response, ScrollArea, Shape, TextEdit, Ui};

use super::field::{PADDING_X, paint_field};
use crate::theme::RADIUS;

/// Room above and below the text, so its first line does not touch the
/// box's top.
const PADDING_Y: i8 = 10;

/// `edit` in a filled box as wide as the room, named `name`, that
/// scrolls both ways once its text is taller than `max_height`. The
/// caller keeps the text, font and whether it can be changed.
pub fn text_box(ui: &mut Ui, edit: TextEdit<'_>, name: &str, max_height: f32) -> Response {
    text_box_at(ui, edit, name, max_height, None)
}

/// [`text_box`] scrolled so the character at `shown`, counted in
/// characters, sits in the middle of the box.
pub fn text_box_at(
    ui: &mut Ui,
    edit: TextEdit<'_>,
    name: &str,
    max_height: f32,
    shown: Option<usize>,
) -> Response {
    // The fill depends on the focus the text only reports once drawn.
    let fill_slot = ui.painter().add(Shape::Noop);
    let framed = Frame::NONE
        .inner_margin(Margin::symmetric(PADDING_X, PADDING_Y))
        .show(ui, |ui| {
            ScrollArea::both()
                .id_salt(("text_box", name))
                .max_height(max_height)
                .show(ui, |ui| {
                    let output = edit
                        .frame(Frame::NONE)
                        .desired_width(f32::INFINITY)
                        .show(ui);

                    if let Some(index) = shown {
                        let place = output.galley.pos_from_cursor(CCursor::new(index));
                        ui.scroll_to_rect(
                            place.translate(output.galley_pos.to_vec2()),
                            Some(Align::Center),
                        );
                    }

                    output.response.response
                })
                .inner
        });

    let response = framed.inner;

    // The ring goes round the whole box, not the text that holds the
    // focus inside it.
    paint_field(
        ui,
        fill_slot,
        framed.response.rect,
        RADIUS,
        response.has_focus(),
    );

    let label = name.to_owned();
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label(label));

    response
}
