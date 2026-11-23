//! What a page shows instead of its list: nothing to show, or a list
//! that could not be read with a way to read it again. Kirigami's
//! `PlaceholderMessage`: one monochrome icon, a heading, an explanation and
//! a button, all centred.

use egui::{Label, Response, Sense, Ui, vec2};

use super::buttons::pill_button;
use super::{ButtonKind, rich};
use crate::fonts::{BODY, SECTION_TITLE, Weight};
use crate::icons::{self, Icon};
use crate::tokens;

/// Kirigami's default placeholder icon, `iconSizes.huge` times 1.5.
const ICON: f32 = 96.0;
/// `largeSpacing` between the items of the column.
const LARGE_SPACING: f32 = 8.0;
/// The button's `topMargin` on top of the column's spacing, a `gridUnit`.
const ACTION_GAP: f32 = 26.0;

/// `icon` above `title` and `body`, centred in the page, as Kirigami's
/// `PlaceholderMessage` lays them out. The icon only adorns the title, so
/// screen readers skip it.
pub fn empty_state(ui: &mut Ui, icon: Icon, title: &str, body: &str) -> Response {
    let tokens = tokens::current(ui.ctx());
    ui.add_space(tokens.sizes.block_gap);
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(ICON, ICON), Sense::hover());
        let theme = icons::theme_in_use(ui.ctx());
        icon.image(theme.as_deref(), tokens.dim)
            .fit_to_exact_size(vec2(ICON, ICON))
            .paint_at(ui, rect);

        let heading = (!title.is_empty()).then(|| {
            ui.add_space(LARGE_SPACING);
            ui.add(
                Label::new(rich(
                    ui,
                    title,
                    Weight::Bold,
                    SECTION_TITLE,
                    tokens.secondary,
                ))
                .wrap(),
            )
        });

        if !body.is_empty() {
            ui.add_space(LARGE_SPACING);
            ui.add(Label::new(rich(ui, body, Weight::Regular, BODY, tokens.secondary)).wrap());
        }

        heading.unwrap_or_else(|| ui.allocate_response(vec2(0.0, 0.0), Sense::hover()))
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
    ui.add_space(ACTION_GAP);
    ui.vertical_centered(|ui| pill_button(ui, action, ButtonKind::Soft))
        .inner
}
