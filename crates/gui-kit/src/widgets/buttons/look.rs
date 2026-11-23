//! Darkly's colours and shadows for push and flat buttons, as pure
//! functions of the tokens and the pointer, so the look is tested without
//! egui.

use egui::{Color32, Shadow};

use super::super::press::Pointer;
use crate::tokens::{Tokens, darkly};

/// Darkly's `Button_MarginWidth`, on each side of a push button's text.
const BUTTON_MARGIN: f32 = 6.0;
/// Darkly's `Button_MinWidth`, for the text and its margins.
const BUTTON_MIN_WIDTH: f32 = 80.0;
/// Darkly's `ButtonWidth` (1) and the frame on both sides (4).
const BUTTON_EXTRA_WIDTH: f32 = 5.0;
/// Darkly's shadow radii are blurred as egui's width, which spans the
/// penumbra on both sides, so the blur is twice the radius.
pub(in crate::widgets) const BLUR_PER_RADIUS: u8 = 2;

/// Darkly's push button face (`buttonBackgroundColor`, darklyhelper.cpp
/// 255-275, and `renderButtonFrame`, 811-885): the focus fills it, the
/// hover lightens the button colour by 5 %, lightened twice from the focus
/// fill when both, and a press darkens the focus colour by 10 %.
pub(in crate::widgets) fn push_fill(
    tokens: &Tokens,
    base: Color32,
    pointer: Pointer,
    focused: bool,
) -> Color32 {
    match (pointer, focused) {
        (Pointer::Pressed, _) => tokens.button_pressed,
        (Pointer::Hovered, true) => darkly::lighter(darkly::lighter(tokens.focus_fill, 1.15), 1.02),
        (Pointer::Hovered, false) => darkly::lighter(base, 1.05),
        (Pointer::Rest, true) => tokens.focus_fill,
        (Pointer::Rest, false) => base,
    }
}

/// The blur radius of a push button's shadow (darklyhelper.cpp:829-838).
pub(in crate::widgets) const fn push_blur(pointer: Pointer, focused: bool) -> u8 {
    match (pointer, focused) {
        (Pointer::Pressed, _) => 2,
        (Pointer::Hovered, _) | (Pointer::Rest, true) => 6,
        (Pointer::Rest, false) => 3,
    }
}

/// The shadow under a raised push button: one pixel down, black at 40
/// of 255 (`control_shadow`), blurred as [`push_blur`] says.
pub(in crate::widgets) const fn push_shadow(
    tokens: &Tokens,
    pointer: Pointer,
    focused: bool,
) -> Shadow {
    Shadow {
        offset: [0, 1],
        blur: push_blur(pointer, focused) * BLUR_PER_RADIUS,
        spread: 0,
        color: tokens.control_shadow,
    }
}

/// The ripple of a pressed push button: the focus colour darker(200) at
/// half opacity (darklyhelper.cpp:884).
pub(in crate::widgets) fn press_overlay(tokens: &Tokens) -> Color32 {
    darkly::darker(tokens.decoration_focus, 2.0).gamma_multiply(0.5)
}

/// A flat icon button's fill (`toolButtonColor`, darklyhelper.cpp
/// 285-319): the text colour at a tenth under the pointer, at a fifth
/// while pressed, the focus colour when only the focus holds it.
pub(in crate::widgets) fn tool_fill(
    tokens: &Tokens,
    pointer: Pointer,
    focused: bool,
) -> Option<Color32> {
    match (pointer, focused) {
        (Pointer::Pressed, _) => Some(tokens.text.gamma_multiply(0.2)),
        (Pointer::Hovered, _) => Some(tokens.text.gamma_multiply(0.1)),
        (Pointer::Rest, true) => Some(tokens.focus_fill),
        (Pointer::Rest, false) => None,
    }
}

/// A push button's width for `content` (its text, and its icon and gap)
/// and at least `at_least`, from `pushButtonSizeFromContents`
/// (darklystyle.cpp:3608-3662).
pub(in crate::widgets) fn push_width(content: f32, at_least: f32) -> f32 {
    ((2.0_f32.mul_add(BUTTON_MARGIN, content)).max(BUTTON_MIN_WIDTH) + BUTTON_EXTRA_WIDTH)
        .max(at_least)
}
