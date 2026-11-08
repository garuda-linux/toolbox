use egui::epaint::{RectShape, Shape};
use egui::{Color32, CornerRadius, Id, LayerId, Order, Rect, TextureId, pos2, vec2};
use egui_kittest::{Harness, HarnessBuilder};

use crate::frost::{backdrop, live, reserve};
use crate::testing::harness;

fn band() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 80.0))
}

fn red_block(ctx: &egui::Context) {
    let page = ctx.layer_painter(LayerId::background());
    page.rect_filled(
        Rect::from_min_size(pos2(0.0, 80.0), vec2(400.0, 120.0)),
        0.0,
        Color32::RED,
    );
}

fn frosted_band(ctx: &egui::Context) {
    let surface = ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("surface")));
    backdrop(&surface, band(), CornerRadius::ZERO);
}

fn textured(harness: &Harness<'_>) -> Vec<RectShape> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            Shape::Rect(rect) if matches!(rect.fill_texture_id(), TextureId::User(_)) => {
                Some(rect.clone())
            }
            Shape::Rect(_)
            | Shape::Noop
            | Shape::Vec(_)
            | Shape::Circle(_)
            | Shape::Ellipse(_)
            | Shape::LineSegment {
                points: _,
                stroke: _,
            }
            | Shape::Path(_)
            | Shape::Text(_)
            | Shape::Mesh(_)
            | Shape::QuadraticBezier(_)
            | Shape::CubicBezier(_)
            | Shape::Band(_)
            | Shape::Callback(_) => None,
        })
        .collect()
}

#[test]
fn the_test_harness_has_the_live_blur() {
    let harness = harness(400.0, 200.0).build_ui(|ui| assert!(live(ui.ctx())));
    drop(harness);
}

#[test]
fn a_backdrop_is_filled_with_a_texture_of_its_own() {
    let mut harness = harness(400.0, 200.0).build_ui(|ui| {
        let ctx = ui.ctx();
        red_block(ctx);
        frosted_band(ctx);
    });

    harness.run();

    let blurred = textured(&harness);
    assert_eq!(blurred.len(), 1, "{blurred:?}");
    assert_eq!(blurred.first().map(|rect| rect.rect), Some(band()));
}

#[test]
fn a_surface_drawn_before_the_content_under_it_still_blurs_it() {
    let mut harness = harness(400.0, 200.0).build_ui(|ui| {
        let ctx = ui.ctx();
        frosted_band(ctx);
        red_block(ctx);
    });

    harness.run();

    // The block starts below the band: only the blur carries its red up
    // into the band's lowest rows.
    let image = harness.render().expect("the wgpu test renderer draws");
    let [red, _, blue, _] = image.get_pixel(200, 76).0;
    assert!(red > blue.saturating_add(40), "red {red} blue {blue}");
}

#[test]
fn without_the_live_blur_a_surface_keeps_no_place() {
    let harness = HarnessBuilder::default().build_ui(|ui| {
        let ctx = ui.ctx();
        let painter = ctx.layer_painter(LayerId::background());
        assert!(reserve(&painter).is_none());
    });

    drop(harness);
}

/// egui-wgpu samples native textures as gamma values, an sRGB target
/// would blur too dark.
#[test]
fn an_srgb_target_keeps_the_tint_alone() {
    let mut state = egui_kittest::wgpu::create_render_state(
        egui_kittest::wgpu::default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );

    state.target_format = egui_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb;

    let ctx = egui::Context::default();
    crate::frost::install(&ctx, state);

    assert!(!live(&ctx));
}
