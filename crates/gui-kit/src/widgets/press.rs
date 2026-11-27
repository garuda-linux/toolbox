//! What the pointer does to a control this frame, so every painted control
//! answers a press alike. A disabled control fades once, egui's
//! `disabled_alpha` over its own colours, so a Ghost button stays transparent.

use egui::{Color32, CornerRadius, Rect, Response, Ui};

use super::focus::focus_from_keyboard;
use crate::motion;
use crate::theme::RADIUS;
use crate::tokens::Tokens;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pointer {
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

    /// Like [`Self::of`], with keyboard focus shown as a hover. A click
    /// focuses without showing it, so only a key shows it.
    pub(super) fn of_keyboard(ui: &Ui, response: &Response) -> Self {
        let pointer = Self::of(response);
        if pointer == Self::Rest && focus_from_keyboard(ui, response) && ui.is_enabled() {
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
            Self::Hovered => tokens.button_hover,
            Self::Pressed => tokens.button_pressed,
        }
    }

    /// Hovered or pressed: the fill under the pointer shows.
    pub(super) const fn is_under(self) -> bool {
        !matches!(self, Self::Rest)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ItemLook {
    pub fill: Color32,
    pub frame: Color32,
}

/// A list or sidebar item as Darkly's item view paints it: the selection
/// fill when selected, `item_hover` under the pointer, `tint_hover` when
/// both. Darkly draws no outline on an item, so the frame stays clear.
pub(super) const fn item_look(tokens: &Tokens, selected: bool, pointer: Pointer) -> ItemLook {
    let fill = match (selected, pointer) {
        (false, Pointer::Rest) => Color32::TRANSPARENT,
        (false, _) => tokens.item_hover,
        (true, Pointer::Rest) => tokens.selection,
        (true, _) => tokens.tint_hover,
    };

    ItemLook {
        fill,
        frame: Color32::TRANSPARENT,
    }
}

/// Paints `look` on `rect` with Darkly's item radius.
pub(super) fn paint_item(ui: &Ui, rect: Rect, look: ItemLook) {
    ui.painter()
        .rect_filled(rect, CornerRadius::same(RADIUS), look.fill);
}

/// `color` as a control in `ui` draws it. A disabled Ui already fades its
/// painter, so only a control switched off on its own in an enabled Ui
/// fades here, by the same `disabled_alpha`.
pub(super) fn faded(ui: &Ui, color: Color32, enabled: bool) -> Color32 {
    if enabled || !ui.is_enabled() {
        color
    } else {
        ui.visuals().disable(color)
    }
}

/// How far the hover fill has faded in under `pointer`: 0 at rest, 1 under
/// the pointer, eased over the hover fade. Every control that fades its
/// hover uses this one rule.
pub(super) fn hover_amount(ui: &Ui, response: &Response, pointer: Pointer) -> f32 {
    let seconds = motion::duration(ui.ctx(), motion::HOVER_FADE);
    ui.ctx()
        .animate_bool_with_time(response.id.with("hover"), pointer.is_under(), seconds)
}

/// The hover fill of a row or item: `tokens.item_hover` faded in as a pill
/// button fades, and one step deeper at once while pressed.
pub(super) fn hover_fill(
    ui: &Ui,
    response: &Response,
    pointer: Pointer,
    tokens: &Tokens,
) -> Color32 {
    if pointer == Pointer::Pressed {
        return tokens.button_pressed;
    }

    tokens
        .item_hover
        .gamma_multiply(hover_amount(ui, response, pointer))
}
