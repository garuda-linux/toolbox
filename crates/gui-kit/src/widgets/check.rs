//! The box and the path of a check mark, which a check row paints and
//! strokes in its slot, as Darkly's check box does.

use egui::{Color32, Pos2, Rect, vec2};

use super::press::Pointer;
use crate::tokens::{Tokens, darkly};

/// The side of the box the mark's points are in, Darkly's frame of 16 px.
const BOX_SIDE: f32 = 16.0;

/// Darkly's `renderCheckBox` mark in its 16 px box.
const MARK: [(f32, f32); 3] = [(5.0, 8.5), (7.0, 11.0), (12.0, 5.0)];

/// Darkly's shadow under the mark, black at 100 of 255.
const MARK_SHADOW_OPACITY: f32 = 100.0 / 255.0;

/// The mark's path within `rect`, the box, cut at `drawn` (0 to 1) of its
/// length.
pub(super) fn partial(rect: Rect, drawn: f32) -> Vec<Pos2> {
    let scale = rect.width() / BOX_SIDE;
    let [start, knee, end] = MARK.map(|(x, y)| rect.min + vec2(x * scale, y * scale));

    let strokes = [(start, knee), (knee, end)];
    let lengths = strokes.map(|(from, to)| from.distance(to));

    let mut left = lengths.iter().sum::<f32>() * drawn.clamp(0.0, 1.0);
    let mut points = Vec::with_capacity(MARK.len());
    points.push(start);
    for ((from, to), length) in strokes.into_iter().zip(lengths) {
        if left >= length {
            points.push(to);
            left -= length;
            continue;
        }

        if left > 0.0 && length > 0.0 {
            points.push(from.lerp(to, left / length));
        }

        break;
    }

    points
}

/// The fill of a check box as Darkly's `renderCheckBox` paints it. Darkly
/// draws no frame.
pub(super) fn box_fill(tokens: &Tokens, on: bool, pointer: Pointer) -> Color32 {
    let base = if on { tokens.accent_tint } else { tokens.input };
    if pointer.is_under() {
        darkly::lighter(base, 1.1)
    } else {
        base
    }
}

pub(super) fn mark_shadow(tokens: &Tokens) -> Color32 {
    tokens.shadow_shade.gamma_multiply(MARK_SHADOW_OPACITY)
}
