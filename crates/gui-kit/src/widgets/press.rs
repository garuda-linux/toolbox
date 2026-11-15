//! What the pointer does to a control this frame, so every painted
//! control answers a press alike: a fill one step deeper while the button
//! is down on it, and no scale.
//!
//! One look for a disabled control: egui's fade of a disabled Ui
//! (`disabled_alpha`) over the control's own colours. Nothing swaps a
//! colour or fades a second time, so a Ghost button stays transparent.

use egui::{Color32, Response, Ui};

use crate::motion;
use crate::tokens::Tokens;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pointer {
    Rest,
    Hovered,
    Pressed,
}

impl Pointer {
    pub fn of(response: &Response) -> Self {
        if response.is_pointer_button_down_on() {
            Self::Pressed
        } else if response.hovered() {
            Self::Hovered
        } else {
            Self::Rest
        }
    }

    /// Like [`Self::of`], with keyboard focus shown as a hover, for the
    /// surfaces that mark their focus by their fill.
    pub fn of_focusable(response: &Response) -> Self {
        let pointer = Self::of(response);
        if pointer == Self::Rest && response.has_focus() {
            Self::Hovered
        } else {
            pointer
        }
    }

    /// The fill of a surface that rests on `rest`: the hover fill under
    /// the pointer, one step deeper while pressed.
    pub const fn fill(self, tokens: &Tokens, rest: Color32) -> Color32 {
        match self {
            Self::Rest => rest,
            Self::Hovered => tokens.hover,
            Self::Pressed => tokens.pressed,
        }
    }

    /// Hovered or pressed: the fill under the pointer shows.
    pub const fn is_under(self) -> bool {
        !matches!(self, Self::Rest)
    }
}

/// `color` as a control in `ui` draws it. A disabled Ui already fades its
/// painter, so only a control switched off on its own in an enabled Ui
/// fades here, by the same `disabled_alpha`.
pub fn faded(ui: &Ui, color: Color32, enabled: bool) -> Color32 {
    if enabled || !ui.is_enabled() {
        color
    } else {
        ui.visuals().disable(color)
    }
}

/// How far the hover fill has faded in under `pointer`: 0 at rest, 1 under
/// the pointer, eased over the hover fade. Every control that fades its
/// hover uses this one rule.
pub fn hover_amount(ui: &Ui, response: &Response, pointer: Pointer) -> f32 {
    let seconds = motion::duration(ui.ctx(), motion::HOVER_FADE);
    ui.ctx()
        .animate_bool_with_time(response.id.with("hover"), pointer.is_under(), seconds)
}

/// The hover fill of a row or item: `tokens.hover` faded in as a pill
/// button fades, and one step deeper at once while pressed.
pub fn hover_fill(ui: &Ui, response: &Response, pointer: Pointer, tokens: &Tokens) -> Color32 {
    if pointer == Pointer::Pressed {
        return tokens.pressed;
    }

    tokens
        .hover
        .gamma_multiply(hover_amount(ui, response, pointer))
}
