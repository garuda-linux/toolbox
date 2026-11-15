//! An on/off switch, the control of every toggle row. It speaks as a
//! check box, so screen readers announce its state.

use egui::accesskit::Toggled;
use egui::{Color32, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, pos2, vec2};

use super::focus_ring;
use super::press::faded;
use crate::motion;
use crate::tokens::{self, DIM_CONTRAST, Tokens, contrast};

pub(super) const SIZE: Vec2 = vec2(40.0, 24.0);
const TRACK_RADIUS: u8 = 12;
const KNOB_RADIUS: f32 = 9.0;

/// A click on an enabled one reports `clicked`, the caller flips its state.
pub fn switch(ui: &mut Ui, on: bool, label: &str, enabled: bool) -> Response {
    tristate_switch(ui, Some(on), label, enabled)
}

/// A [`switch`] whose state may not be known yet, `None`: the knob rests
/// in the middle of a neutral track and screen readers hear "mixed".
pub fn tristate_switch(ui: &mut Ui, on: Option<bool>, label: &str, enabled: bool) -> Response {
    // A disabled Ui, such as a row that waits for a plan, rests it too.
    let enabled = enabled && ui.is_enabled();
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };

    let (rect, response) = ui.allocate_exact_size(SIZE, sense);
    if let Some(on) = on {
        response.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, on, label));
    } else {
        response.widget_info(|| WidgetInfo::labeled(Role::CheckBox, enabled, label));
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_toggled(Toggled::Mixed));
    }

    if ui.is_rect_visible(rect) {
        paint(ui, &response, on, enabled);
    }

    response
}

fn paint(ui: &Ui, response: &Response, on: Option<bool>, enabled: bool) {
    let tokens = tokens::current(ui.ctx());
    let rect = response.rect;

    // A flip the keyboard asked for jumps, a click slides.
    if let Some(on) = on
        && motion::keyboard_marked(ui)
    {
        motion::settle(ui.ctx(), response.id, on);
    }

    // How far the knob went, and how much of the accent the track took.
    let (place, lit) = on.map_or((0.5, 0.0), |on| {
        let progress = ui.ctx().animate_bool_responsive(response.id, on);
        (progress, progress)
    });

    let hovered = response.hovered();
    let dark = ui.visuals().dark_mode;
    let off = match (dark, hovered) {
        (true, true) | (false, false) => tokens.pressed,
        (true, false) => tokens.hover,
        (false, true) => tokens.track_hover,
    };

    let on_fill = if hovered {
        tokens.accent_hover
    } else {
        tokens.accent
    };

    let fade = |color| faded(ui, color, enabled);
    let track = off.lerp_to_gamma(on_fill, lit);

    // The off track is too close to the card to read as a control in
    // either theme. The rim melts into the accent as the switch turns on.
    let rim_colour = tokens.rim.lerp_to_gamma(on_fill, lit);
    let rim = Stroke::new(1.0, fade(rim_colour));

    let painter = ui.painter();
    painter.rect(rect, TRACK_RADIUS, fade(track), rim, StrokeKind::Inside);

    let inset = SIZE.y / 2.0;
    let knob_x = egui::lerp((rect.left() + inset)..=(rect.right() - inset), place);
    let (off_knob, on_knob) = knobs(&tokens, dark);
    let knob = off_knob.lerp_to_gamma(on_knob, lit);

    // A light knob on a light track needs an edge, most of all faded, as
    // a disabled one is. On the accent it melts away.
    let edge = if dark {
        Stroke::NONE
    } else {
        Stroke::new(1.0, fade(tokens.rim.lerp_to_gamma(knob, lit)))
    };

    let centre = pos2(knob_x, rect.center().y);
    painter.circle(centre, KNOB_RADIUS, fade(knob), edge);

    focus_ring(ui, response, TRACK_RADIUS);
}

/// The knob off and on. One colour in both states while it reads at 3:1
/// on the accent and its hover shade, else the on knob takes `on_accent`.
pub(super) fn knobs(tokens: &Tokens, dark: bool) -> (Color32, Color32) {
    let off = if dark { tokens.text } else { tokens.raised };
    let holds = [tokens.accent, tokens.accent_hover]
        .into_iter()
        .all(|fill| contrast(off, fill) >= DIM_CONTRAST);

    (off, if holds { off } else { tokens.on_accent })
}
