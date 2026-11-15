//! What a page shows instead of its list: nothing to show, or a list
//! that could not be read with a way to read it again.

use egui::{Label, Response, Ui, vec2};

use super::icon_tile::TileSize;
use super::status_tile::status_tile;
use super::{ButtonKind, Tone, pill_button, rich};
use crate::fonts::{BODY, Weight};
use crate::icons::Icon;
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

/// An [`empty_state`] with a Soft button named `action` centred under
/// `body`, for a page that could not be read. Returns the button's
/// response.
pub fn empty_state_action(
    ui: &mut Ui,
    icon: Icon,
    title: &str,
    body: &str,
    action: &str,
) -> Response {
    _ = empty_state(ui, icon, title, body);
    action_button(ui, action)
}

/// An [`empty_state`] that shows `action` as a Soft button under `body`
/// only when there is one. Returns the button's response.
pub fn empty_state_with(
    ui: &mut Ui,
    icon: Icon,
    title: &str,
    body: &str,
    action: Option<&str>,
) -> Option<Response> {
    _ = empty_state(ui, icon, title, body);
    action.map(|action| action_button(ui, action))
}

fn action_button(ui: &mut Ui, action: &str) -> Response {
    ui.add_space(tokens::current(ui.ctx()).sizes.card_gap);
    ui.vertical_centered(|ui| pill_button(ui, action, ButtonKind::Soft))
        .inner
}
