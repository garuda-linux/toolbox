//! A box of several lines of text, filled as the one-line fields are but
//! with rounded corners, since round ends on a tall box would cut its
//! text.

use egui::text::CCursor;
use egui::{Align, AsIdSalt, Frame, Margin, Response, ScrollArea, Shape, TextEdit, Ui};

use super::field::{FRAME_WIDTH, paint_field};
use crate::theme::{FIELD_RADIUS, show_scroll};

/// Room above and below the text, the line edit's 7 px frame, so its first
/// line does not touch the box's top.
const PADDING_Y: i8 = FRAME_WIDTH;

/// `edit` in a filled box as wide as the room, named `name`, that
/// scrolls both ways once its text is taller than `max_height`. The
/// caller keeps the text, font and whether it can be changed.
pub fn text_box(
    ui: &mut Ui,
    (salt, name): (impl AsIdSalt, &str),
    edit: TextEdit<'_>,
    max_height: f32,
) -> Response {
    text_box_at(ui, (salt, name), edit, max_height, None)
}

/// [`text_box`] scrolled so the character at `shown`, counted in
/// characters, sits in the middle of the box.
pub fn text_box_at(
    ui: &mut Ui,
    (salt, name): (impl AsIdSalt, &str),
    edit: TextEdit<'_>,
    max_height: f32,
    shown: Option<usize>,
) -> Response {
    // The fill depends on the focus the text only reports once drawn.
    let fill_slot = ui.painter().add(Shape::Noop);
    let framed = Frame::NONE
        .inner_margin(Margin::symmetric(FRAME_WIDTH, PADDING_Y))
        .show(ui, |ui| {
            show_scroll(
                ui,
                ScrollArea::both()
                    .id_salt(("text_box", salt))
                    .max_height(max_height)
                    .auto_shrink([false, true]),
                |ui| {
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
                },
            )
            .inner
        });

    let response = framed.inner;

    // The focus halo goes round the whole box, not the text that holds the
    // focus inside it.
    paint_field(
        ui,
        fill_slot,
        framed.response.rect,
        FIELD_RADIUS,
        response.has_focus(),
    );

    response.accessible_name(name)
}
