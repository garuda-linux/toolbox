//! The path of a check mark, which a check row strokes in its box.

use egui::{Pos2, Rect, vec2};

/// The mark's corners within its square, from the short stroke's start
/// to the long stroke's end.
const POINTS: [(f32, f32); 3] = [(0.18, 0.52), (0.42, 0.74), (0.84, 0.28)];

/// The mark's path within `rect`, cut at `drawn` (0 to 1) of its length.
pub(super) fn partial(rect: Rect, drawn: f32) -> Vec<Pos2> {
    let [start, knee, end] =
        POINTS.map(|(x, y)| rect.min + vec2(x * rect.width(), y * rect.height()));

    let strokes = [(start, knee), (knee, end)];
    let lengths = strokes.map(|(from, to)| from.distance(to));

    let mut left = lengths.iter().sum::<f32>() * drawn.clamp(0.0, 1.0);
    let mut points = Vec::with_capacity(POINTS.len());
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
