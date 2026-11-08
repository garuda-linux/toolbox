//! What the pointer does to a control this frame, so every painted
//! control answers a press alike: a fill one step deeper while the button
//! is down on it, and no scale.

use egui::{Color32, Response};

use crate::tokens::Tokens;

/// How much of a disabled control's parts shows.
const DISABLED_OPACITY: f32 = 0.45;

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

/// `color` as an enabled or a disabled control draws it.
pub fn faded(color: Color32, enabled: bool) -> Color32 {
    if enabled {
        color
    } else {
        color.gamma_multiply(DISABLED_OPACITY)
    }
}
