//! An on/off switch, the control of every toggle row. It speaks as a
//! check box, so screen readers announce its state.

use egui::{Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, pos2, vec2};

use super::focus_ring;
use super::press::faded;
use crate::{motion, tokens};

pub(super) const SIZE: Vec2 = vec2(40.0, 22.0);
const TRACK_RADIUS: u8 = 11;
const KNOB_RADIUS: f32 = 8.0;
/// How much of its rim a disabled switch shows in light themes: enough
/// to be seen on a white card, below an enabled rim's 3:1.
const LIGHT_DISABLED_RIM: f32 = 0.75;

/// A click on an enabled one reports `clicked`, the caller flips its state.
pub fn switch(ui: &mut Ui, on: bool, label: &str, enabled: bool) -> Response {
    // A disabled Ui, such as a row that waits for a plan, rests it too.
    let enabled = enabled && ui.is_enabled();
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };

    let (rect, response) = ui.allocate_exact_size(SIZE, sense);
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, on, label));

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let tokens = tokens::current(ui.ctx());

    // A flip the keyboard asked for jumps, a click slides.
    if motion::keyboard_marked(ui) {
        motion::settle(ui.ctx(), response.id, on);
    }

    let progress = ui.ctx().animate_bool_responsive(response.id, on);

    let hovered = response.hovered();
    let (off, off_knob) = if ui.visuals().dark_mode {
        (
            if hovered {
                tokens.pressed
            } else {
                tokens.hover
            },
            tokens.text,
        )
    } else {
        let track = if hovered {
            tokens.track_hover
        } else {
            tokens.pressed
        };

        (track, tokens.raised)
    };

    let on_fill = if hovered {
        tokens.accent_hover
    } else {
        tokens.accent
    };

    let fade = |color| faded(color, enabled);
    let track = off.lerp_to_gamma(on_fill, progress);

    // The off track is too close to the card to read as a control in
    // either theme. The rim melts into the accent as the switch turns on.
    let rim_colour = tokens.rim.lerp_to_gamma(on_fill, progress);
    let rim = if enabled || ui.visuals().dark_mode {
        fade(rim_colour)
    } else {
        // A faded rim and track all but vanish on a white card, so the
        // rim of a disabled switch keeps more of its colour in light.
        rim_colour.gamma_multiply(LIGHT_DISABLED_RIM)
    };

    let rim = Stroke::new(1.0, rim);

    let painter = ui.painter();
    painter.rect(rect, TRACK_RADIUS, fade(track), rim, StrokeKind::Inside);

    let inset = SIZE.y / 2.0;
    let knob_x = egui::lerp((rect.left() + inset)..=(rect.right() - inset), progress);
    let knob = off_knob.lerp_to_gamma(tokens.on_accent, progress);
    painter.circle_filled(pos2(knob_x, rect.center().y), KNOB_RADIUS, fade(knob));

    focus_ring(ui, &response, TRACK_RADIUS);
    response
}
