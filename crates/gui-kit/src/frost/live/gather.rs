//! Runs without a GPU, on the shapes egui paints below a frosted surface.

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use egui::emath::TSTransform;
use egui::epaint::ClippedShape;
use egui::layers::GraphicLayers;
use egui::{LayerId, Order, Rect, Shape, TextureId};

/// egui's orders, bottom first (its own list is private).
const PAINT_ORDER: [Order; 5] = [
    Order::Background,
    Order::Middle,
    Order::Foreground,
    Order::Tooltip,
    Order::Debug,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    pub layer: LayerId,
    pub slot: usize,
}

pub struct Scene<'frame> {
    pub order: &'frame [LayerId],
    pub transforms: &'frame HashMap<LayerId, TSTransform>,
    pub blurred: &'frame [TextureId],
}

/// The shapes that paint before `place` and touch `area`, moved so that
/// `area`'s corner is the origin. Paint callbacks, text and the backdrops
/// of other surfaces are left out: a callback cannot run twice in a frame,
/// and what a backdrop shows is gathered anyway while its texture may
/// change after this blur was kept.
pub fn gather(
    layers: &GraphicLayers,
    scene: &Scene<'_>,
    place: Place,
    area: Rect,
) -> Vec<ClippedShape> {
    collect(layers, scene, place, area, None)
}

/// [`gather`], but only what lies within `bounds`, and no shadow. The
/// header band shows what scrolls under it alone: neither the sidebar
/// beside it nor a control below it, nor a shadow smeared up from there.
pub fn gather_within(
    layers: &GraphicLayers,
    scene: &Scene<'_>,
    place: Place,
    area: Rect,
    bounds: Rect,
) -> Vec<ClippedShape> {
    collect(layers, scene, place, area, Some(bounds))
}

fn is_shadow(shape: &Shape) -> bool {
    matches!(shape, Shape::Rect(rect) if rect.blur_width > 0.0)
}

fn collect(
    layers: &GraphicLayers,
    scene: &Scene<'_>,
    place: Place,
    area: Rect,
    bounds: Option<Rect>,
) -> Vec<ClippedShape> {
    let mut shapes = Vec::new();
    let cut = bounds.map_or(area, |bounds| bounds.intersect(area));
    let shadows = bounds.is_none();
    let to_origin = TSTransform::from_translation(-area.min.to_vec2());

    for layer in sequence(scene.order, place.layer) {
        let Some(list) = layers.get(layer) else {
            continue;
        };

        let own = layer == place.layer;
        let entries = list.all_entries();
        let count = if own {
            place.slot.min(entries.len())
        } else {
            entries.len()
        };

        let transform = scene.transforms.get(&layer).copied().unwrap_or_default();

        // Culled before the clone: a dialog over a long page would copy
        // the whole page, meshes included, every frame.
        shapes.extend(
            entries
                .take(count)
                // Small text samples its neighbours in the font atlas, whose
                // places change between runs. Blurred it is a smudge anyway.
                .filter(|clipped| !matches!(clipped.shape, Shape::Callback(_) | Shape::Text(_)))
                .filter(|clipped| !scene.blurred.contains(&clipped.shape.texture_id()))
                .filter(|clipped| shadows || !is_shadow(&clipped.shape))
                .filter(|clipped| {
                    (transform * clipped.clip_rect).intersects(cut)
                        && (transform * clipped.shape.visual_bounding_rect()).intersects(cut)
                })
                .map(|clipped| {
                    let mut clipped = clipped.clone();
                    clipped.transform(transform);

                    // Past its reach the blur would smear in what lies
                    // farther out. Past its bounds, what lies beside it.
                    clipped.clip_rect = clipped.clip_rect.intersect(cut);
                    clipped.transform(to_origin);
                    clipped
                }),
        );

        if own {
            return shapes;
        }
    }

    shapes
}

/// The layers in the order egui paints them: by order first, the listed
/// areas of an order, then the layers no area lists, as panels paint into
/// the background layer. Only the page layer and the surface's own are
/// known among those.
fn sequence(order: &[LayerId], own: LayerId) -> Vec<LayerId> {
    let unlisted = [LayerId::background(), own];
    PAINT_ORDER
        .into_iter()
        .flat_map(|level| {
            let listed = order
                .iter()
                .copied()
                .filter(move |layer| layer.order == level);

            let extra = unlisted
                .into_iter()
                .filter(move |layer| layer.order == level && !order.contains(layer));

            listed.chain(extra)
        })
        .fold(Vec::new(), |mut layers, layer| {
            // At most a few dozen layers, a scan beats a set here.
            if !layers.contains(&layer) {
                layers.push(layer);
            }

            layers
        })
}
