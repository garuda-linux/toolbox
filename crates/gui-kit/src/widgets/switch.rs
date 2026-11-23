//! An on/off switch, the control of every toggle row. It draws the QQC2
//! desktop style's switch in Darkly colours: a track in the line edit's
//! colours with the solid highlight over the part the knob has passed, and
//! a knob in the field colour. It speaks as a check box, so screen readers
//! announce its state.

use egui::accesskit::Toggled;
use egui::{
    Color32, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, pos2, vec2,
};

use super::focus::focus_from_keyboard;
use super::press::faded;
use crate::motion;
use crate::theme::GRID_UNIT;
use crate::tokens::{self, Tokens, mix};

/// `SwitchIndicator.qml` of qqc2-desktop-style: as tall as Kirigami's
/// grid unit and twice as wide.
pub(super) const SIZE: Vec2 = vec2(2.0 * GRID_UNIT, GRID_UNIT);
/// `floor(height / 6)` above and below the track.
pub(super) const TRACK_MARGIN: f32 = 3.0;
/// The knob is as tall as the switch and round.
const KNOB_RADIUS: f32 = GRID_UNIT / 2.0;
/// Half the track's height, so its ends are round.
const TRACK_RADIUS: u8 = 6;
/// Breeze's `frameContrast`, how far the frame moves from the fill toward
/// the text.
const FRAME_CONTRAST: f32 = 0.2;

/// A click reports `clicked`, the caller flips its state. A disabled
/// switch sits inside `add_enabled_ui`, which fades it.
pub fn switch(ui: &mut Ui, on: bool, name: &str) -> Response {
    tristate_switch(ui, Some(on), name)
}

/// A [`switch`] whose state may not be known yet, `None`: the knob rests
/// in the middle of a neutral track and screen readers hear "mixed".
pub fn tristate_switch(ui: &mut Ui, on: Option<bool>, name: &str) -> Response {
    let enabled = ui.is_enabled();
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };

    let (rect, response) = ui.allocate_exact_size(SIZE, sense);
    if let Some(on) = on {
        response.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, on, name));
    } else {
        response.widget_info(|| WidgetInfo::labeled(Role::CheckBox, enabled, name));
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_toggled(Toggled::Mixed));
    }

    if ui.is_rect_visible(rect) {
        paint(ui, &response, on, enabled);
    }

    response
}

/// The track's fill and frame when `on`, and with `lit` the frame in the
/// highlight. The highlight laid over a checked track is
/// [`highlight_colour`].
pub(super) const fn track_colours(tokens: &Tokens, on: bool, lit: bool) -> (Color32, Color32) {
    let track = tokens.input;
    let rim = if on || lit { tokens.accent } else { tokens.rim };

    (track, rim)
}

/// The highlight over the track as far as the knob has passed, solid when
/// on and fading in with the knob.
pub(super) const fn highlight_colour(tokens: &Tokens) -> Color32 {
    tokens.accent_tint
}

/// The knob's fill and frame. It takes the highlight when lit.
pub(super) fn knob_colours(tokens: &Tokens, lit: bool) -> (Color32, Color32) {
    let rim = if lit { tokens.accent } else { frame(tokens) };
    (tokens.field, rim)
}

fn frame(tokens: &Tokens) -> Color32 {
    mix(tokens.field, tokens.text, FRAME_CONTRAST)
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

    let place = on.map_or(0.5, |on| ui.ctx().animate_bool_responsive(response.id, on));
    let checked = on == Some(true);
    let lit = enabled && (response.hovered() || focus_from_keyboard(ui, response));

    let fade = |color| faded(ui, color, enabled);
    let (track, rim) = track_colours(&tokens, checked, lit);

    let track_rect = Rect::from_min_max(
        pos2(rect.left(), rect.top() + TRACK_MARGIN),
        pos2(rect.right(), rect.bottom() - TRACK_MARGIN),
    );

    let knob_left = egui::lerp(rect.left()..=(rect.right() - SIZE.y), place);
    let highlight = Rect::from_min_max(track_rect.min, pos2(knob_left + SIZE.y, track_rect.max.y));

    let painter = ui.painter();
    painter.rect(
        track_rect,
        TRACK_RADIUS,
        fade(track),
        Stroke::new(1.0, fade(rim)),
        StrokeKind::Inside,
    );

    if on.is_some() {
        let colour = highlight_colour(&tokens).gamma_multiply(place);
        painter.rect_filled(highlight, TRACK_RADIUS, fade(colour));
    }

    let (knob, knob_rim) = knob_colours(&tokens, lit);
    let centre = pos2(knob_left + KNOB_RADIUS, rect.center().y);
    painter.circle(
        centre,
        KNOB_RADIUS - 0.5,
        fade(knob),
        Stroke::new(1.0, fade(knob_rim)),
    );
}
