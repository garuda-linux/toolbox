//! A backdrop that shows only what lies within its own bounds, as the
//! header band does.

use std::collections::HashMap;

use egui::epaint::{RectShape, Shadow};
use egui::layers::GraphicLayers;
use egui::{Color32, Rect, Shape, pos2, vec2};

use super::{add, colours, menu, page, square};
use crate::frost::live::gather::{Place, Scene, gather, gather_within};

fn place() -> Place {
    Place {
        layer: menu(),
        slot: 0,
    }
}

fn soft_shadow() -> Shape {
    let shadow = Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::BLACK,
    };

    shadow
        .as_shape(Rect::from_min_size(pos2(5.0, 5.0), vec2(20.0, 20.0)), 4.0)
        .into()
}

#[test]
fn a_shadow_is_left_out_within_but_kept_around() {
    let mut layers = GraphicLayers::default();
    add(&mut layers, page(), soft_shadow());
    add(&mut layers, page(), square(0.0, 0.0, Color32::RED));
    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(50.0, 50.0));

    let within = gather_within(
        &layers,
        &Scene {
            order: &[],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place(),
        area,
        area,
    );

    let around = gather(
        &layers,
        &Scene {
            order: &[],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place(),
        area,
    );

    assert_eq!(colours(&within), vec![Color32::RED]);
    assert_eq!(around.len(), 2, "a floating surface still blurs the shadow");
}

#[test]
fn what_lies_beside_the_bounds_is_left_out() {
    let mut layers = GraphicLayers::default();

    // Inside the reach, outside the band: the sidebar, a button below.
    add(&mut layers, page(), square(0.0, 20.0, Color32::BLUE));
    add(&mut layers, page(), square(40.0, 20.0, Color32::RED));
    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 60.0));
    let band = Rect::from_min_max(pos2(30.0, 0.0), pos2(100.0, 40.0));

    let shapes = gather_within(
        &layers,
        &Scene {
            order: &[],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place(),
        area,
        band,
    );

    assert_eq!(colours(&shapes), vec![Color32::RED]);
}

#[test]
fn a_shape_reaching_out_of_the_bounds_is_cut_to_them() {
    let mut layers = GraphicLayers::default();
    let card: Shape = RectShape::filled(
        Rect::from_min_max(pos2(40.0, 20.0), pos2(80.0, 90.0)),
        0.0,
        Color32::RED,
    )
    .into();

    add(&mut layers, page(), card);
    let area = Rect::from_min_size(pos2(10.0, 0.0), vec2(100.0, 100.0));
    let band = Rect::from_min_max(pos2(30.0, 0.0), pos2(110.0, 40.0));

    let shapes = gather_within(
        &layers,
        &Scene {
            order: &[],
            transforms: &HashMap::new(),
            blurred: &[],
        },
        place(),
        area,
        band,
    );

    // Moved to the area's corner like the shape itself.
    let clip = shapes.first().map(|clipped| clipped.clip_rect);
    assert_eq!(
        clip,
        Some(Rect::from_min_max(pos2(20.0, 0.0), pos2(100.0, 40.0)))
    );
}
