use egui::{Rect, Ui, vec2};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::line;

#[cfg(test)]
mod tests;

const RESERVED_COUNT: &str = "99";

pub(super) fn badge(ui: &mut Ui, count: &str) -> egui::Response {
    let tokens = tokens::current(ui.ctx());
    let size = tokens.sizes.badge_size;
    let galley = line(ui, count, Weight::Bold, SMALL, tokens.on_accent);
    let reserved = line(ui, RESERVED_COUNT, Weight::Bold, SMALL, tokens.on_accent)
        .size()
        .x;

    let width = (galley.size().x.max(reserved) + tokens.sizes.badge_padding).max(size);

    // As tall as the buttons beside it: placed first in its row, a slot of
    // its own height would be centred before the buttons make the row
    // taller and end up above their middle.
    let (slot, response) = ui.allocate_exact_size(
        vec2(width, toolbox_gui_kit::theme::PILL_HEIGHT),
        egui::Sense::hover(),
    );

    let rect = Rect::from_center_size(slot.center(), vec2(width, size));
    ui.painter().rect_filled(rect, size / 2.0, tokens.accent);
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        tokens.on_accent,
    );
    response
}
