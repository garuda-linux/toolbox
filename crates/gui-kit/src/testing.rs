//! The kittest harness every UI test starts from: the wgpu test renderer
//! and the fallback look, set before the first pass so the first layout
//! already uses them.

mod caps;

pub use caps::{cap_off_centre, capitals_top, deep_descent_font, last_baseline_within, test_fonts};
use egui_kittest::HarnessBuilder;

use crate::fonts::{FontFile, Weight};

/// A harness builder of `width` × `height` points in the fallback look.
#[must_use = "a builder does nothing until a harness is built from it"]
pub fn harness<State>(width: f32, height: f32) -> HarnessBuilder<State> {
    harness_with_font(width, height, None)
}

/// [`harness`] with `font` as the desktop's font at every weight, the
/// bundled fonts behind it, or the bundled fonts alone for `None`.
#[must_use = "a builder does nothing until a harness is built from it"]
pub fn harness_with_font<State>(
    width: f32,
    height: f32,
    font: Option<FontFile>,
) -> HarnessBuilder<State> {
    // One render state for the snapshots and the live blur, as eframe
    // shares its own.
    let state = egui_kittest::wgpu::create_render_state(
        egui_kittest::wgpu::default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );

    let renderer = egui_kittest::wgpu::WgpuTestRenderer::from_render_state(state.clone());

    HarnessBuilder::default()
        .renderer(renderer)
        .with_size(egui::vec2(width, height))
        // No painted mouse pointer, as it would hide parts of the snapshots.
        .with_render_cursor(false)
        // The live blur renders a change twice before it settles.
        .with_max_steps(8)
        .with_context_setup(move |ctx| {
            crate::install(ctx);
            if let Some(font) = font {
                let weights = Weight::ALL.map(|weight| FontFile {
                    weight: weight.css(),
                    ..font.clone()
                });

                ctx.set_fonts(crate::fonts::definitions(&weights, None));
            }

            crate::frost::install(ctx, state);
        })
}

/// The colour the last frame painted `text` in, if it painted it.
pub fn text_colour<State>(
    harness: &egui_kittest::Harness<'_, State>,
    text: &str,
) -> Option<egui::Color32> {
    painted_text(harness, text)?
        .galley
        .job
        .sections
        .first()
        .map(|section| section.format.color)
}

/// The shape the last frame painted `text` with, if it painted it.
pub fn painted_text<State>(
    harness: &egui_kittest::Harness<'_, State>,
    text: &str,
) -> Option<egui::epaint::TextShape> {
    painted_text_within(harness, text, egui::Rect::EVERYTHING)
}

/// The shape the last frame painted `text` with inside `rect`, for a
/// text painted twice, such as a dropdown's chosen entry and its button.
pub fn painted_text_within<State>(
    harness: &egui_kittest::Harness<'_, State>,
    text: &str,
    rect: egui::Rect,
) -> Option<egui::epaint::TextShape> {
    painted(harness).into_iter().find_map(|shape| {
        if let egui::Shape::Text(painted) = shape
            && painted.galley.text() == text
            && rect.contains(painted.pos + painted.galley.rect.center().to_vec2())
        {
            Some(painted.clone())
        } else {
            None
        }
    })
}

/// Where the last frame painted a rectangle filled with `fill`, the first
/// one if several.
pub fn filled_rect<State>(
    harness: &egui_kittest::Harness<'_, State>,
    fill: egui::Color32,
) -> Option<egui::Rect> {
    filled_rects(harness, fill).first().copied()
}

/// The stroke widths the last frame drew around rectangles that hold
/// `point`, outermost first.
pub fn rims_around<State>(
    harness: &egui_kittest::Harness<'_, State>,
    point: egui::Pos2,
) -> Vec<f32> {
    rects(harness)
        .filter(|painted| painted.rect.contains(point))
        .map(|painted| painted.stroke.width)
        .collect()
}

/// Every rectangle the last frame stroked in `colour`, with the stroke's
/// width.
pub fn rims_in<State>(
    harness: &egui_kittest::Harness<'_, State>,
    colour: egui::Color32,
) -> Vec<(egui::Rect, f32)> {
    rects(harness)
        .filter(|painted| painted.stroke.color == colour && painted.stroke.width > 0.0)
        .map(|painted| (painted.rect, painted.stroke.width))
        .collect()
}

/// The stroke colours the last frame drew around rectangles that hold
/// `point`, outermost first, those of no width left out.
pub fn rim_colours_around<State>(
    harness: &egui_kittest::Harness<'_, State>,
    point: egui::Pos2,
) -> Vec<egui::Color32> {
    rects(harness)
        .filter(|painted| painted.rect.contains(point) && painted.stroke.width > 0.0)
        .map(|painted| painted.stroke.color)
        .collect()
}

/// The fill of the topmost rectangle the last frame painted under `point`,
/// for the strength of a button.
pub fn fill_under<State>(
    harness: &egui_kittest::Harness<'_, State>,
    point: egui::Pos2,
) -> Option<egui::Color32> {
    rects(harness)
        .filter(|painted| {
            painted.rect.contains(point) && painted.fill != egui::Color32::TRANSPARENT
        })
        .last()
        .map(|painted| painted.fill)
}

/// Every place the last frame painted a rectangle filled with `fill`.
pub fn filled_rects<State>(
    harness: &egui_kittest::Harness<'_, State>,
    fill: egui::Color32,
) -> Vec<egui::Rect> {
    rects(harness)
        .filter(|painted| painted.fill == fill)
        .map(|painted| painted.rect)
        .collect()
}

/// The horizontal lines the last frame drew in `colour`, as their two
/// ends.
pub fn lines_in<State>(
    harness: &egui_kittest::Harness<'_, State>,
    colour: egui::Color32,
) -> Vec<[egui::Pos2; 2]> {
    painted(harness)
        .into_iter()
        .filter_map(|shape| {
            if let egui::Shape::LineSegment { points, stroke } = shape
                && stroke.color == colour
            {
                Some(*points)
            } else {
                None
            }
        })
        .collect()
}

/// The fills of the circles the last frame painted, in paint order.
pub fn circle_fills<State>(harness: &egui_kittest::Harness<'_, State>) -> Vec<egui::Color32> {
    painted(harness)
        .into_iter()
        .filter_map(|shape| {
            if let egui::Shape::Circle(circle) = shape {
                Some(circle.fill)
            } else {
                None
            }
        })
        .collect()
}

/// Every place the last frame painted a rectangle with a texture.
pub fn textured_rects<State>(harness: &egui_kittest::Harness<'_, State>) -> Vec<egui::Rect> {
    rects(harness)
        .filter(|painted| painted.brush.is_some())
        .map(|painted| painted.rect)
        .collect()
}

/// Every place the last frame painted a glyph tinted `tint`: a textured
/// rectangle filled with that colour.
pub fn tinted_rects<State>(
    harness: &egui_kittest::Harness<'_, State>,
    tint: egui::Color32,
) -> Vec<egui::Rect> {
    rects(harness)
        .filter(|painted| painted.brush.is_some() && painted.fill == tint)
        .map(|painted| painted.rect)
        .collect()
}

/// Where the last frame drew a live blur: rects textured from a texture
/// egui does not manage, as the frost's own targets are.
pub fn blurred_rects<State>(harness: &egui_kittest::Harness<'_, State>) -> Vec<egui::Rect> {
    rects(harness)
        .filter(|painted| matches!(painted.fill_texture_id(), egui::TextureId::User(_)))
        .map(|painted| painted.rect)
        .collect()
}

/// Every shape the last frame painted, those inside a `Shape::Vec`
/// included, in paint order.
pub(crate) fn painted<'frame, State>(
    harness: &'frame egui_kittest::Harness<'_, State>,
) -> Vec<&'frame egui::Shape> {
    fn walk<'shape>(shape: &'shape egui::Shape, found: &mut Vec<&'shape egui::Shape>) {
        if let egui::Shape::Vec(shapes) = shape {
            for inner in shapes {
                walk(inner, found);
            }
        } else {
            found.push(shape);
        }
    }

    let mut found = Vec::new();
    for clipped in &harness.output().shapes {
        walk(&clipped.shape, &mut found);
    }

    found
}

fn rects<'frame, State>(
    harness: &'frame egui_kittest::Harness<'_, State>,
) -> impl Iterator<Item = &'frame egui::epaint::RectShape> {
    painted(harness).into_iter().filter_map(|shape| {
        if let egui::Shape::Rect(painted) = shape {
            Some(painted)
        } else {
            None
        }
    })
}
