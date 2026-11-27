//! The light along a dark surface's top edge, as from a lamp above. It
//! follows both top corners and fades out down them, so the surface reads
//! raised without a rim round it. A push button takes Darkly's even line
//! instead, see [`paint_button_top_light`].

use std::f32::consts::{FRAC_PI_2, PI};

use egui::emath::GuiRounding as _;
use egui::epaint::{Mesh, PathShape, PathStroke, Tessellator};
use egui::{CornerRadius, Painter, Pos2, Rect, Shape, Theme, pos2};

use crate::tokens;

const WIDTH: f32 = 1.0;
/// Points on each quarter circle, enough for a smooth curve at a
/// popover's radius.
const ARC_STEPS: u8 = 8;

/// Light themes keep their rim and get none.
pub fn paint_top_light(painter: &Painter, rect: Rect, corners: impl Into<CornerRadius>) {
    let ctx = painter.ctx();
    if ctx.theme() != Theme::Dark || !rect.is_positive() {
        return;
    }

    let corners = corners.into();
    let colour = tokens::current(ctx).top_light;
    let (edge, most) = top_edge(painter, rect);
    let left = (f32::from(corners.nw) - WIDTH / 2.0).clamp(0.0, most);
    let right = (f32::from(corners.ne) - WIDTH / 2.0).clamp(0.0, most);
    let points = top_corners(edge, left, right);

    let (top, fall) = (edge.top(), left.max(right));
    let stroke = PathStroke::new_uv(WIDTH, move |_, at| {
        let down = if fall > 0.0 {
            ((at.y - top) / fall).clamp(0.0, 1.0)
        } else {
            0.0
        };

        colour.gamma_multiply(1.0 - down)
    });

    paint_path(painter, points, stroke);
}

/// Darkly's `topHighlight` on a push button's face (darklyhelper.cpp:857):
/// one pixel of the light end along the top edge, even down both top
/// corners and gone before the sides. Clear on a light window.
pub(super) fn paint_button_top_light(painter: &Painter, rect: Rect, radius: u8) {
    let colour = tokens::current(painter.ctx()).button_top_light;
    if colour.a() == 0 || !rect.is_positive() {
        return;
    }

    let (edge, most) = top_edge(painter, rect);
    let corner = (f32::from(radius) - WIDTH / 2.0).clamp(0.0, most);
    let points = top_corners(edge, corner, corner);

    paint_path(painter, points, PathStroke::new(WIDTH, colour));
}

/// The rect a top light runs along, and the longest a corner of it may be.
/// As egui rounds a rect, no corner is wider than half the side it ends.
fn top_edge(painter: &Painter, rect: Rect) -> (Rect, f32) {
    let edge = rect
        .round_to_pixels(painter.pixels_per_point())
        .shrink(WIDTH / 2.0);

    let most = edge.width().min(edge.height()) / 2.0;
    (edge, most)
}

fn top_corners(edge: Rect, left: f32, right: f32) -> Vec<Pos2> {
    let mut points = arc(pos2(edge.left() + left, edge.top() + left), left, PI);
    points.extend(arc(
        pos2(edge.right() - right, edge.top() + right),
        right,
        PI + FRAC_PI_2,
    ));
    points
}

/// Tessellated here into a mesh, which compares by value: a path with a
/// colour function never equals itself, so the frost under a lit menu
/// would never settle.
fn paint_path(painter: &Painter, points: Vec<Pos2>, stroke: PathStroke) {
    let options = painter.ctx().tessellation_options(|options| *options);
    let mut tessellator = Tessellator::new(painter.pixels_per_point(), options, [1, 1], Vec::new());
    let mut mesh = Mesh::default();
    tessellator.tessellate_path(&PathShape::line(points, stroke), &mut mesh);
    painter.add(Shape::mesh(mesh));
}

fn arc(centre: Pos2, radius: f32, start: f32) -> Vec<Pos2> {
    if radius <= 0.0 {
        return vec![centre];
    }

    (0..=ARC_STEPS)
        .map(|step| {
            let angle = (f32::from(step) / f32::from(ARC_STEPS)).mul_add(FRAC_PI_2, start);
            centre + radius * egui::vec2(angle.cos(), angle.sin())
        })
        .collect()
}
