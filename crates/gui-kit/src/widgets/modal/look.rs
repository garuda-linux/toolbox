//! Kirigami's `Dialog` look: its corners and frame. The shadow is Darkly's
//! popup shadow, painted by the dialog itself, and the menus keep their own.

use egui::{CornerRadius, Frame, Margin, Stroke, Style};

use crate::theme::CARD_RADIUS;
use crate::tokens::{Tokens, mix};

/// Kirigami's frame contrast: how far the edge moves from the fill toward the text.
const FRAME_CONTRAST: f32 = 0.2;

/// The frame around a dialog's content, `padding` from its edge on every side.
pub(super) fn frame(style: &Style, tokens: &Tokens, padding: i8) -> Frame {
    let edge = mix(tokens.card, tokens.text, FRAME_CONTRAST);
    Frame::popup(style)
        .inner_margin(Margin::same(padding))
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .stroke(Stroke::new(1.0, edge))
        .shadow(egui::Shadow::NONE)
}
