//! What a page shows instead of its list: nothing to show, or a list
//! that could not be read with a way to read it again.

use egui::{Label, Response, Ui};

use super::icon_tile::{TileSize, icon_tile};
use super::status_tile::status_tile;
use super::{ButtonKind, Tone, pill_button, rich};
use crate::fonts::{BODY, Weight};
use crate::icons::Icon;
use crate::tokens;

/// A title a little larger than body text, as a heading of the state.
const TITLE_SCALE: f32 = 1.15;

/// `icon` above `title` and `body`, centred in the page, on a large tile:
/// a state glyph as its status mark, any other icon in the page's hue.
/// The icon only adorns the title, so screen readers skip it.
pub fn empty_state(ui: &mut Ui, icon: Icon, title: &str, body: &str) -> Response {
    let tokens = tokens::current(ui.ctx());
    ui.add_space(tokens.sizes.block_gap);
    ui.vertical_centered(|ui| {
        if let Some(tone) = Tone::of_state_glyph(icon) {
            _ = status_tile(ui, TileSize::Large, tone);
        } else {
            _ = icon_tile(ui, TileSize::Large, icon);
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
