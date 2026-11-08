//! The GPU side of the live blur: shapes rendered small into a texture of
//! their own by eframe's renderer, then blurred by a separable Gaussian in
//! two widths. Each surface keeps its own target, so several can show in
//! one frame.

#[cfg(test)]
mod tests;

mod pipeline;

use egui::epaint::{ClippedPrimitive, ClippedShape, Primitive};
use egui::{Color32, Context, TextureId};
use egui_wgpu::{RenderState, ScreenDescriptor, wgpu};
pub use pipeline::Pipeline;

/// The blur passes: from texture, to texture, step in texels. Horizontal
/// and vertical, then both again twice as wide.
const PASSES: [(usize, usize, [f32; 2]); 4] = [
    (0, 1, [1.0, 0.0]),
    (1, 2, [0.0, 1.0]),
    (2, 1, [2.0, 0.0]),
    (1, 2, [0.0, 2.0]),
];

/// The largest side of a target, in texels, so a huge window never asks
/// the GPU for more than it gives.
const MOST_TEXELS: f32 = 4096.0;

/// The blur's own resolution: a quarter of the window's.
const DOWNSCALE: f32 = 4.0;

pub struct Target {
    size: [u32; 2],
    views: [wgpu::TextureView; 3],
    steps: [wgpu::Buffer; 4],
    /// Each blur pass's inputs, built when the size changes, as they
    /// change with nothing else.
    groups: Vec<wgpu::BindGroup>,
    texture: Option<TextureId>,
    last: Option<(Vec<ClippedShape>, [u32; 2], Color32)>,
    /// The last render had every texture it used in full, so its result
    /// may stay.
    settled: bool,
}

impl Target {
    pub fn new(state: &RenderState) -> Self {
        let steps = std::array::from_fn(|_| {
            state.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("toolbox-frost step"),
                size: 16,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });

        Self {
            size: [1, 1],
            views: views(state, [1, 1]),
            steps,
            groups: Vec::new(),
            texture: None,
            last: None,
            settled: false,
        }
    }

    /// The texture to draw, after rendering and blurring `shapes` (moved
    /// to the origin of an area of `points`) over `background`. The work
    /// is skipped when nothing changed since the last frame.
    pub fn blur(
        &mut self,
        job: &Job<'_>,
        shapes: Vec<ClippedShape>,
        points: egui::Vec2,
    ) -> Option<TextureId> {
        let scale = job.ctx.pixels_per_point() / DOWNSCALE;
        let size = [texels(points.x * scale)?, texels(points.y * scale)?];
        let fresh = (shapes, size, job.background);

        let same = self.last.as_ref() == Some(&fresh);
        if self.texture.is_some() && same && self.settled {
            return self.texture;
        }

        let (drawn, _, background) = fresh;
        let state = job.state;
        let mut renderer = state.renderer.write();

        if size != self.size || self.texture.is_none() {
            self.resize(job.pipeline, state, &mut renderer, size);
        }

        let meshes = job.ctx.tessellate(drawn.clone(), scale);

        // A texture egui made this frame (an icon, a picture) reaches the
        // renderer only when the frame paints, after this render. Its shapes
        // wait for the next render, and a result without them never stays.
        let (meshes, complete) = drawable(meshes, |id| renderer.texture(id).is_some());

        let screen = ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: scale,
        };

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut buffers =
            renderer.update_buffers(&state.device, &state.queue, &mut encoder, &meshes, &screen);

        {
            let [first, ..] = &self.views;
            let attachments = [Some(pass_into(first, background))];
            let mut pass = encoder
                .begin_render_pass(&describe(&attachments))
                .forget_lifetime();

            renderer.render(&mut pass, &meshes, &screen);
        }

        drop(renderer);

        self.passes(job.pipeline, &mut encoder);
        buffers.push(encoder.finish());
        state.queue.submit(buffers);

        // Textures and glyphs egui made this frame reach the renderer only
        // when the frame paints, after this render. So only a second render
        // of the same shapes may stay.
        self.settled = complete && same;

        // Content that animates brings the next frame itself, and asking
        // for one every frame would never let the window rest.
        if !self.settled && !job.ctx.has_requested_repaint() {
            job.ctx.request_repaint();
        }

        self.last = complete.then_some((drawn, size, background));
        self.texture
    }

    fn resize(
        &mut self,
        pipeline: &Pipeline,
        state: &RenderState,
        renderer: &mut egui_wgpu::Renderer,
        size: [u32; 2],
    ) {
        self.size = size;
        self.views = views(state, size);
        self.groups = self.groups(pipeline, state);
        let [.., result] = &self.views;
        match self.texture {
            Some(id) => renderer.update_egui_texture_from_wgpu_texture(
                &state.device,
                result,
                wgpu::FilterMode::Linear,
                id,
            ),
            None => {
                self.texture = Some(renderer.register_native_texture(
                    &state.device,
                    result,
                    wgpu::FilterMode::Linear,
                ));
            }
        }
    }

    /// The texture this target last drew into, to free when it goes.
    pub const fn texture(&self) -> Option<TextureId> {
        self.texture
    }

    fn groups(&self, pipeline: &Pipeline, state: &RenderState) -> Vec<wgpu::BindGroup> {
        let texel = [1.0 / size_f32(self.size[0]), 1.0 / size_f32(self.size[1])];

        PASSES
            .into_iter()
            .zip(&self.steps)
            .filter_map(|((from, _, [x, y]), step)| {
                let source = self.views.get(from)?;
                let bytes: Vec<u8> = [x * texel[0], y * texel[1], 0.0, 0.0]
                    .iter()
                    .flat_map(|value| value.to_le_bytes())
                    .collect();

                state.queue.write_buffer(step, 0, &bytes);

                Some(state.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("toolbox-frost"),
                    layout: &pipeline.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(source),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: step.as_entire_binding(),
                        },
                    ],
                }))
            })
            .collect()
    }

    fn passes(&self, pipeline: &Pipeline, encoder: &mut wgpu::CommandEncoder) {
        for ((_, to, _), group) in PASSES.into_iter().zip(&self.groups) {
            let Some(target) = self.views.get(to) else {
                continue;
            };

            let attachments = [Some(pass_into(target, Color32::TRANSPARENT))];
            let mut pass = encoder.begin_render_pass(&describe(&attachments));
            pass.set_pipeline(&pipeline.render);
            pass.set_bind_group(0, group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

pub struct Job<'frame> {
    pub ctx: &'frame Context,
    pub state: &'frame RenderState,
    pub pipeline: &'frame Pipeline,
    pub background: Color32,
}

fn views(state: &RenderState, size: [u32; 2]) -> [wgpu::TextureView; 3] {
    let [width, height] = size;
    std::array::from_fn(|_| {
        state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("toolbox-frost"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: state.target_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    })
}

fn pass_into(view: &wgpu::TextureView, clear: Color32) -> wgpu::RenderPassColorAttachment<'_> {
    let [red, green, blue, alpha] = clear.to_normalized_gamma_f32().map(f64::from);
    wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        depth_slice: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
                r: red,
                g: green,
                b: blue,
                a: alpha,
            }),
            store: wgpu::StoreOp::Store,
        },
    }
}

fn describe<'pass>(
    attachments: &'pass [Option<wgpu::RenderPassColorAttachment<'pass>>],
) -> wgpu::RenderPassDescriptor<'pass> {
    wgpu::RenderPassDescriptor {
        label: Some("toolbox-frost"),
        color_attachments: attachments,
        ..wgpu::RenderPassDescriptor::default()
    }
}

fn drawable(
    meshes: Vec<ClippedPrimitive>,
    known: impl Fn(&TextureId) -> bool,
) -> (Vec<ClippedPrimitive>, bool) {
    let total = meshes.len();
    let kept: Vec<ClippedPrimitive> = meshes
        .into_iter()
        .filter(|mesh| match &mesh.primitive {
            Primitive::Mesh(mesh) => known(&mesh.texture_id),
            Primitive::Callback(_) => true,
        })
        .collect();

    let complete = kept.len() == total;
    (kept, complete)
}

/// A side in texels, at most [`MOST_TEXELS`]. None for an empty side or
/// one that is not a number, which no texture can have.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a positive number clamped to 1..=4096 first, so the cast is exact"
)]
fn texels(length: f32) -> Option<u32> {
    (length > 0.0).then(|| length.ceil().clamp(1.0, MOST_TEXELS) as u32)
}

fn size_f32(texels: u32) -> f32 {
    u16::try_from(texels).map_or(MOST_TEXELS, f32::from)
}
