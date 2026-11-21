use std::f32::consts::PI;

use egui::epaint::Mesh;
use egui::{Color32, Painter, Rect, Shape, Ui, pos2};

use crate::{motion, theme};

/// Room between the band's columns. Its light changes slowly enough that
/// the mesh needs no more.
const COLUMN: f32 = 2.0;

/// One phase for the whole list, so the band reads as one sweep.
pub(super) fn band_middle(ui: &Ui, rect: Rect) -> Option<f32> {
    if theme::reduced_motion(ui.ctx())
        || frozen(ui.ctx())
        || !ui.is_rect_visible(rect)
        || !ui.input(motion::window_in_sight)
    {
        return None;
    }

    ui.ctx().request_repaint_after(motion::SHIMMER_FRAME);
    let period = f64::from(motion::SHIMMER_PERIOD);

    #[expect(
        clippy::cast_possible_truncation,
        clippy::as_conversions,
        reason = "the phase only needs a float's precision"
    )]
    let phase = (ui.input(|input| input.time).rem_euclid(period) / period) as f32;
    let band = motion::SHIMMER_BAND;
    Some(phase.mul_add(rect.width() + band, rect.left() - band / 2.0))
}

/// The band keeps half a point inside the bar's edge, so the bar's own
/// smooth edge shows round it.
pub(super) fn paint_shine(painter: &Painter, bar: Rect, radius: f32, middle: f32, light: Color32) {
    let half = motion::SHIMMER_BAND / 2.0;
    let inner = bar.shrink(0.5);
    let (from, to) = (
        (middle - half).max(inner.left()),
        (middle + half).min(inner.right()),
    );

    if from >= to {
        return;
    }

    let radius = (radius - 0.5).clamp(0.0, inner.height() / 2.0);
    let mut mesh = Mesh::default();
    let mut x = from;
    let mut column = 0_u32;
    loop {
        let (top, bottom) = span(inner, radius, x);
        let colour = light.gamma_multiply(strength((x - middle) / half));
        mesh.colored_vertex(pos2(x, top), colour);
        mesh.colored_vertex(pos2(x, bottom), colour);
        if column > 0 {
            let at = column * 2;
            mesh.add_triangle(at - 2, at - 1, at);
            mesh.add_triangle(at - 1, at, at + 1);
        }

        if x >= to {
            break;
        }

        column += 1;
        x = (x + COLUMN).min(to);
    }

    painter.add(Shape::mesh(mesh));
}

fn span(bar: Rect, radius: f32, x: f32) -> (f32, f32) {
    let from_end = (x - bar.left()).min(bar.right() - x).max(0.0);
    let inset = if from_end < radius {
        let across = radius - from_end;
        radius - radius.mul_add(radius, -(across * across)).max(0.0).sqrt()
    } else {
        0.0
    };

    (bar.top() + inset, bar.bottom() - inset)
}

fn strength(offset: f32) -> f32 {
    if offset.abs() >= 1.0 {
        0.0
    } else {
        0.5_f32.mul_add((PI * offset).cos(), 0.5)
    }
}

#[cfg(any(test, feature = "testing"))]
fn frozen(ctx: &egui::Context) -> bool {
    crate::testing::shimmer_frozen(ctx)
}

#[cfg(not(any(test, feature = "testing")))]
const fn frozen(_ctx: &egui::Context) -> bool {
    false
}
