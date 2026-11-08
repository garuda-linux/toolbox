//! What a page shows instead of its list: nothing to show, or a list that
//! did not load with a way to try again.

use egui::{Align, Label, Layout, Response, Ui, vec2};

use super::icon_tile::TileSize;
use super::status_tile::status_tile;
use super::{ButtonKind, Tone, pill_button, rich, split_row};
use crate::fonts::{BODY, Weight};
use crate::icons::Icon;
use crate::theme::PILL_HEIGHT;
use crate::tokens;
use crate::ui_ext::icon_image;

/// The empty state's icon is this many times a body line.
const ICON_SCALE: f32 = 2.2;
/// A title a little larger than body text, as a heading of the state.
const TITLE_SCALE: f32 = 1.15;

/// `icon` above `title` and `body`, centred in the page. A state glyph
/// shows as its status mark, any other icon dimmed. The icon only
/// adorns the title, so screen readers skip it.
pub fn empty_state(ui: &mut Ui, icon: Icon, title: &str, body: &str) -> Response {
    let tokens = tokens::current(ui.ctx());
    ui.add_space(tokens.sizes.block_gap);
    ui.vertical_centered(|ui| {
        if let Some(tone) = Tone::of_state_glyph(icon) {
            _ = status_tile(ui, TileSize::Row, tone);
        } else {
            let side = ui.text_style_height(&egui::TextStyle::Body) * ICON_SCALE;
            let (rect, _) = ui.allocate_exact_size(vec2(side, side), egui::Sense::hover());
            icon_image(ui, icon, tokens.dim).paint_at(ui, rect);
        }

        ui.add_space(tokens.sizes.control_gap);
        let title = ui.add(
            Label::new(rich(
                ui,
                title,
                Weight::SemiBold,
                BODY * TITLE_SCALE,
                tokens.text,
            ))
            .wrap(),
        );

        ui.add_space(tokens.sizes.tight_gap);

        ui.add(Label::new(rich(ui, body, Weight::Regular, BODY, tokens.secondary)).wrap());
        title
    })
    .inner
}

/// `message` beside the error mark, and a Retry button named
/// `retry_label` at the right. Returns whether Retry was clicked.
pub fn error_row(ui: &mut Ui, message: &str, retry_label: &str) -> bool {
    let tokens = tokens::current(ui.ctx());
    let button_width = super::buttons::pill_width(ui, retry_label);

    split_row(
        ui,
        button_width,
        |ui| {
            // As tall as the button, so the message sits level with it.
            let room = vec2(ui.available_width(), PILL_HEIGHT);
            let layout = Layout::left_to_right(Align::Center);
            ui.allocate_ui_with_layout(room, layout, |ui| {
                _ = status_tile(ui, TileSize::Line, Tone::Error);
                ui.add(Label::new(rich(ui, message, Weight::Medium, BODY, tokens.text)).wrap());
            });
        },
        |ui| pill_button(ui, retry_label, ButtonKind::Soft).clicked(),
    )
    .1
}
