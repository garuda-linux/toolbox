use egui::{Rect, Ui, vec2};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::line;

const BADGE_SIZE: f32 = 22.0;
const BADGE_PADDING: f32 = 12.0;

pub(super) fn badge(ui: &mut Ui, count: &str) -> egui::Response {
    let tokens = tokens::current(ui.ctx());
    let galley = line(ui, count, Weight::Bold, SMALL, tokens.on_accent);
    let width = (galley.size().x + BADGE_PADDING).max(BADGE_SIZE);

    // As tall as the buttons beside it: placed first in its row, a slot of
    // its own height would be centred before the buttons make the row
    // taller and end up above their middle.
    let (slot, response) = ui.allocate_exact_size(
        vec2(width, toolbox_gui_kit::theme::PILL_HEIGHT),
        egui::Sense::hover(),
    );

    let rect = Rect::from_center_size(slot.center(), vec2(width, BADGE_SIZE));
    ui.painter()
        .rect_filled(rect, BADGE_SIZE / 2.0, tokens.accent);
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        tokens.on_accent,
    );
    response
}
