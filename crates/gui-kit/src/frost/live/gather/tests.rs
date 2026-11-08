mod within;

use std::collections::HashMap;

use egui::emath::TSTransform;
use egui::epaint::{ClippedShape, RectShape};
use egui::layers::GraphicLayers;
use egui::{Color32, Id, LayerId, Order, Rect, Shape, pos2, vec2};

use super::{Place, Scene, gather};

fn square(x: f32, y: f32, colour: Color32) -> Shape {
    RectShape::filled(
        Rect::from_min_size(pos2(x, y), vec2(10.0, 10.0)),
        0.0,
        colour,
    )
    .into()
}

fn page() -> LayerId {
    LayerId::background()
}

fn menu() -> LayerId {
    LayerId::new(Order::Foreground, Id::unique("menu"))
}

fn everywhere() -> Rect {
    Rect::EVERYTHING
}

fn add(layers: &mut GraphicLayers, layer: LayerId, shape: Shape) {
    layers.entry(layer).add(everywhere(), shape);
}

fn colours(shapes: &[ClippedShape]) -> Vec<Color32> {
    shapes
        .iter()
        .filter_map(|clipped| {
            if let Shape::Rect(rect) = &clipped.shape {
                Some(rect.fill)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn only_shapes_that_touch_the_area_are_kept() {
    let mut layers = GraphicLayers::default();
    add(&mut layers, page(), square(0.0, 0.0, Color32::RED));
    add(&mut layers, page(), square(500.0, 500.0, Color32::BLUE));
    let place = Place {
        layer: menu(),
        slot: 0,
    };

    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));
    let shapes = gather(
        &layers,
        &Scene {
            order: &[page(), menu()],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place,
        area,
    );

    assert_eq!(colours(&shapes), vec![Color32::RED]);
}

#[test]
fn shapes_move_to_the_areas_corner() {
    let mut layers = GraphicLayers::default();
    add(&mut layers, page(), square(110.0, 220.0, Color32::RED));
    let place = Place {
        layer: menu(),
        slot: 0,
    };

    let area = Rect::from_min_size(pos2(100.0, 200.0), vec2(50.0, 50.0));
    let shapes = gather(
        &layers,
        &Scene {
            order: &[page(), menu()],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place,
        area,
    );

    let Some(Shape::Rect(rect)) = shapes.first().map(|clipped| &clipped.shape) else {
        panic!("the square was not gathered");
    };

    assert_eq!(rect.rect.min, pos2(10.0, 20.0));
}

#[test]
fn a_shape_clipped_away_from_the_area_is_left_out() {
    let mut layers = GraphicLayers::default();
    let clip = Rect::from_min_size(pos2(300.0, 300.0), vec2(10.0, 10.0));
    layers
        .entry(page())
        .add(clip, square(0.0, 0.0, Color32::RED));
    let place = Place {
        layer: menu(),
        slot: 0,
    };

    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));
    let shapes = gather(
        &layers,
        &Scene {
            order: &[page(), menu()],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place,
        area,
    );

    assert_eq!(shapes.len(), 0);
}

#[test]
fn layers_above_the_surface_and_its_own_later_shapes_are_left_out() {
    let mut layers = GraphicLayers::default();
    add(&mut layers, page(), square(0.0, 0.0, Color32::RED));
    add(&mut layers, page(), square(0.0, 0.0, Color32::GREEN));
    add(&mut layers, page(), square(0.0, 0.0, Color32::YELLOW));
    add(&mut layers, menu(), square(0.0, 0.0, Color32::BLUE));
    let place = Place {
        layer: page(),
        slot: 1,
    };

    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));
    let shapes = gather(
        &layers,
        &Scene {
            order: &[page(), menu()],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place,
        area,
    );

    assert_eq!(colours(&shapes), vec![Color32::RED]);
}

#[test]
fn lower_layers_come_first_in_paint_order() {
    let mut layers = GraphicLayers::default();
    let middle = LayerId::new(Order::Middle, Id::unique("sheet"));
    add(&mut layers, middle, square(0.0, 0.0, Color32::GREEN));
    add(&mut layers, page(), square(0.0, 0.0, Color32::RED));
    let place = Place {
        layer: menu(),
        slot: 0,
    };

    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));
    let shapes = gather(
        &layers,
        &Scene {
            order: &[middle, page(), menu()],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place,
        area,
    );

    assert_eq!(colours(&shapes), vec![Color32::RED, Color32::GREEN]);
}

#[test]
fn a_layers_transform_applies_before_the_cut() {
    let mut layers = GraphicLayers::default();
    let moved = LayerId::new(Order::Middle, Id::unique("moved"));
    add(&mut layers, moved, square(0.0, 0.0, Color32::RED));
    let place = Place {
        layer: menu(),
        slot: 0,
    };

    let shift = TSTransform::from_translation(vec2(400.0, 0.0));
    let transforms = HashMap::from([(moved, shift)]);
    let area = Rect::from_min_size(pos2(400.0, 0.0), vec2(50.0, 50.0));
    let scene = Scene {
        order: &[moved, menu()],
        transforms: &transforms,
        blurred: &[],
    };

    let shapes = gather(&layers, &scene, place, area);

    assert_eq!(colours(&shapes), vec![Color32::RED]);
}

#[test]
fn the_page_and_the_surfaces_own_layer_count_though_no_area_lists_them() {
    let mut layers = GraphicLayers::default();
    add(&mut layers, page(), square(0.0, 0.0, Color32::RED));
    add(&mut layers, menu(), square(0.0, 0.0, Color32::BLUE));
    let place = Place {
        layer: menu(),
        slot: 1,
    };

    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));
    let shapes = gather(
        &layers,
        &Scene {
            order: &[],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place,
        area,
    );

    assert_eq!(colours(&shapes), vec![Color32::RED, Color32::BLUE]);
}

#[test]
fn paint_callbacks_are_left_out() {
    let mut layers = GraphicLayers::default();
    let callback = egui::epaint::PaintCallback {
        rect: Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0)),
        callback: std::sync::Arc::new(()),
    };

    add(&mut layers, page(), Shape::Callback(callback));
    add(&mut layers, page(), square(0.0, 0.0, Color32::RED));
    let place = Place {
        layer: menu(),
        slot: 0,
    };

    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));
    let shapes = gather(
        &layers,
        &Scene {
            order: &[page(), menu()],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place,
        area,
    );

    assert_eq!(shapes.len(), 1);
}

#[test]
fn another_surfaces_backdrop_is_left_out() {
    let blurred = egui::TextureId::User(7);
    let backdrop = RectShape::filled(
        Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0)),
        0.0,
        Color32::WHITE,
    )
    .with_texture(blurred, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)));

    let mut layers = GraphicLayers::default();
    add(&mut layers, page(), square(0.0, 0.0, Color32::RED));
    add(&mut layers, page(), backdrop.into());
    let place = Place {
        layer: menu(),
        slot: 0,
    };

    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));
    let shapes = gather(
        &layers,
        &Scene {
            order: &[page(), menu()],
            transforms: &HashMap::new(),
            blurred: &[blurred],
        },
        place,
        area,
    );

    assert_eq!(colours(&shapes), vec![Color32::RED]);
}

#[test]
fn text_is_left_out() {
    let ctx = egui::Context::default();
    let mut first = ctx.run_ui(egui::RawInput::default(), |_| {});
    first.textures_delta.clear();
    let galley = ctx.fonts_mut(|fonts| {
        fonts.layout_no_wrap(
            "Users".to_owned(),
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        )
    });

    let text = Shape::galley(pos2(0.0, 0.0), galley, Color32::WHITE);

    let mut layers = GraphicLayers::default();
    add(&mut layers, page(), square(0.0, 0.0, Color32::RED));
    add(&mut layers, page(), text);
    let place = Place {
        layer: menu(),
        slot: 0,
    };

    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));
    let scene = Scene {
        order: &[page(), menu()],
        transforms: &HashMap::new(),
        blurred: &[],
    };

    let shapes = gather(&layers, &scene, place, area);

    assert_eq!(shapes.len(), 1, "only the square is kept");
}

#[test]
fn a_shape_reaching_past_the_area_is_cut_to_it() {
    let mut layers = GraphicLayers::default();
    add(&mut layers, page(), square(45.0, 45.0, Color32::RED));
    let place = Place {
        layer: menu(),
        slot: 0,
    };

    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));
    let shapes = gather(
        &layers,
        &Scene {
            order: &[page(), menu()],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place,
        area,
    );

    let clip = shapes.first().map(|clipped| clipped.clip_rect);
    assert_eq!(
        clip,
        Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0)))
    );
}
