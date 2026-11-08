//! The live blur under frosted surfaces, on the GPU, in the same frame.
//!
//! A surface keeps a place in its layer for its backdrop while it draws.
//! When the pass ends, everything is painted: the shapes below each place
//! are rendered small by eframe's own wgpu renderer, blurred, and the
//! place is filled with the result. So a surface may draw before the
//! content under it, as a menu does over the rest of its page.

mod gather;
mod gpu;

use std::collections::HashMap;
use std::sync::Arc;

use egui::emath::TSTransform;
use egui::epaint::RectShape;
use egui::layers::ShapeIdx;
use egui::{Color32, Context, CornerRadius, Id, LayerId, Painter, Rect, Shape, TextureId, pos2};
use egui_wgpu::RenderState;
use gather::Place;
use parking_lot::Mutex;

use crate::tokens;

/// How far past a surface the blur reaches for what it smears in, in
/// points. Without it the edges would fade toward the clear colour.
const REACH: f32 = 32.0;

struct Live {
    state: RenderState,
    pipeline: Option<gpu::Pipeline>,
    /// The GPU refused the pipeline, so surfaces keep their tint.
    refused: bool,
    requests: Vec<Request>,
    targets: HashMap<(LayerId, usize), gpu::Target>,
    glyphs: Option<Glyphs>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Glyphs {
    size: [usize; 2],
    fill: f32,
}

impl Live {
    fn restart(&mut self, state: RenderState) {
        // A new state may share the old renderer, which would keep the
        // targets' textures for good.
        self.release_all_but(&[]);
        self.state = state;
        self.pipeline = None;
        self.refused = false;
        self.glyphs = None;
    }

    /// Plain fills sample the atlas's white texel, which moves when the atlas
    /// grows. egui hands over the atlas only when a frame paints, so the blur
    /// hands over the whole atlas itself whenever it changed.
    fn sync_glyphs(&mut self, ctx: &Context) {
        let now = ctx.fonts(|fonts| Glyphs {
            size: fonts.font_image_size(),
            fill: fonts.font_atlas_fill_ratio(),
        });

        if self.glyphs == Some(now) {
            return;
        }

        #[expect(
            clippy::redundant_closure_for_method_calls,
            reason = "the method path does not meet the closure's lifetime"
        )]
        let image = ctx.fonts(|fonts| fonts.image());
        let delta =
            egui::epaint::ImageDelta::full(image, egui::epaint::textures::TextureOptions::LINEAR);

        self.state.renderer.write().update_texture(
            &self.state.device,
            &self.state.queue,
            TextureId::default(),
            &delta,
        );
        self.glyphs = Some(now);
    }

    fn frame(&mut self, ctx: &Context) {
        let requests = std::mem::take(&mut self.requests);
        if requests.is_empty() && self.targets.is_empty() {
            return;
        }

        if !requests.is_empty() {
            self.sync_glyphs(ctx);
        }

        if self.refused {
            return;
        }

        if self.pipeline.is_none() {
            self.pipeline = gpu::Pipeline::new(&self.state);
            self.refused = self.pipeline.is_none();
        }

        let Some(pipeline) = self.pipeline.as_ref() else {
            return;
        };

        let order: Vec<LayerId> = ctx.memory(|memory| memory.layer_ids().collect());
        let transforms: HashMap<LayerId, TSTransform> = order
            .iter()
            .filter_map(|layer| Some((*layer, ctx.layer_transform_to_global(*layer)?)))
            .collect();

        let job = gpu::Job {
            ctx,
            state: &self.state,
            pipeline,
            background: tokens::current(ctx).window,
        };

        let mut used: Vec<(LayerId, usize)> = Vec::with_capacity(requests.len());
        for request in requests {
            let nth = used
                .iter()
                .filter(|(layer, _)| *layer == request.place.layer)
                .count();

            let key = (request.place.layer, nth);
            used.push(key);

            let blurred: Vec<TextureId> = self
                .targets
                .values()
                .filter_map(gpu::Target::texture)
                .collect();

            let target = self
                .targets
                .entry(key)
                .or_insert_with(|| gpu::Target::new(job.state));

            let scene = gather::Scene {
                order: &order,
                transforms: &transforms,
                blurred: &blurred,
            };

            fill(&job, target, &scene, &request);
        }

        self.release_all_but(&used);
    }

    fn release_all_but(&mut self, used: &[(LayerId, usize)]) {
        let gone: Vec<(LayerId, usize)> = self
            .targets
            .keys()
            .filter(|key| !used.contains(key))
            .copied()
            .collect();

        let mut renderer = self.state.renderer.write();
        for key in gone {
            if let Some(texture) = self
                .targets
                .remove(&key)
                .and_then(|target| target.texture())
            {
                renderer.free_texture(&texture);
            }
        }
    }
}

struct Request {
    place: Place,
    clip: Rect,
    rect: Rect,
    corners: CornerRadius,
    /// Only what lies within these is blurred, shadows left out.
    bounds: Option<Rect>,
}

#[derive(Clone, Copy, Debug)]
pub struct Backdrop {
    place: Place,
    clip: Rect,
}

impl Backdrop {
    pub fn fill(self, ctx: &Context, rect: Rect, corners: CornerRadius) {
        self.request(ctx, rect, corners, None);
    }

    fn request(self, ctx: &Context, rect: Rect, corners: CornerRadius, bounds: Option<Rect>) {
        let request = Request {
            place: self.place,
            clip: self.clip,
            rect,
            corners,
            bounds,
        };

        if let Some(installed) = shared(ctx) {
            installed.lock().requests.push(request);
        }
    }
}

/// Without it, frosted surfaces keep their tint alone.
pub fn install(ctx: &Context, state: RenderState) {
    // egui-wgpu samples native textures as gamma values, so a blur made on
    // an sRGB target would come out too dark.
    if state.target_format.is_srgb() {
        tracing::info!(
            "frost: the window's {:?} target is sRGB, surfaces keep their tint without a blur",
            state.target_format
        );
        return;
    }

    if let Some(installed) = shared(ctx) {
        installed.lock().restart(state);
        return;
    }

    let live = Arc::new(Mutex::new(Live {
        state,
        pipeline: None,
        refused: false,
        requests: Vec::new(),
        targets: HashMap::new(),
        glyphs: None,
    }));

    ctx.data_mut(|data| data.insert_temp(live_id(), live));
    ctx.on_end_pass("toolbox-frost", Arc::new(|ui| run(ui.ctx())));
}

pub fn live(ctx: &Context) -> bool {
    shared(ctx).is_some()
}

/// `None` without a live blur.
#[must_use = "the place stays empty until filled"]
pub fn reserve(painter: &Painter) -> Option<Backdrop> {
    shared(painter.ctx())?;
    let ShapeIdx(slot) = painter.add(Shape::Noop);

    Some(Backdrop {
        place: Place {
            layer: painter.layer_id(),
            slot,
        },
        clip: painter.clip_rect(),
    })
}

pub fn backdrop(painter: &Painter, rect: Rect, corners: CornerRadius) {
    if let Some(kept) = reserve(painter) {
        kept.fill(painter.ctx(), rect, corners);
    }
}

/// [`backdrop`] of only what lies under `rect` itself, without shadows: a
/// band over a scrolling page, where what lies beside or below it would
/// smear in as a glow.
pub fn backdrop_within(painter: &Painter, rect: Rect, corners: CornerRadius) {
    if let Some(kept) = reserve(painter) {
        kept.request(painter.ctx(), rect, corners, Some(rect));
    }
}

fn shared(ctx: &Context) -> Option<Arc<Mutex<Live>>> {
    ctx.data(|data| data.get_temp::<Arc<Mutex<Live>>>(live_id()))
}

fn live_id() -> Id {
    Id::unique("toolbox_gui_kit::frost::live")
}

fn run(ctx: &Context) {
    if let Some(installed) = shared(ctx) {
        installed.lock().frame(ctx);
    }
}

fn fill(
    job: &gpu::Job<'_>,
    target: &mut gpu::Target,
    scene: &gather::Scene<'_>,
    request: &Request,
) {
    let to_global = |rect: Rect| {
        scene
            .transforms
            .get(&request.place.layer)
            .map_or(rect, |transform| *transform * rect)
    };

    let area = to_global(request.rect).expand(REACH);
    let bounds = request.bounds.map(to_global);
    let shapes = job.ctx.graphics(|layers| {
        bounds.map_or_else(
            || gather::gather(layers, scene, request.place, area),
            |bounds| gather::gather_within(layers, scene, request.place, area, bounds),
        )
    });

    let Some(texture) = target.blur(job, shapes, area.size()) else {
        return;
    };

    let uv = Rect::from_min_max(
        pos2(REACH / area.width(), REACH / area.height()),
        pos2(1.0 - REACH / area.width(), 1.0 - REACH / area.height()),
    );

    let shape =
        RectShape::filled(request.rect, request.corners, Color32::WHITE).with_texture(texture, uv);

    job.ctx.graphics_mut(|layers| {
        layers.entry(request.place.layer).set(
            ShapeIdx(request.place.slot),
            request.clip,
            shape.into(),
        );
    });
}
