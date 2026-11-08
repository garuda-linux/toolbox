use egui_wgpu::{RenderState, wgpu};

pub struct Pipeline {
    pub(super) render: wgpu::RenderPipeline,
    pub(super) layout: wgpu::BindGroupLayout,
    pub(super) sampler: wgpu::Sampler,
}

impl Pipeline {
    /// None when the driver refuses the shader or the pipeline, so the
    /// surfaces keep their tint instead of the app stopping.
    pub fn new(state: &RenderState) -> Option<Self> {
        Self::with_shader(state, include_str!("../blur.wgsl"))
    }

    pub(super) fn with_shader(state: &RenderState, wgsl: &str) -> Option<Self> {
        let scope = state.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let pipeline = Self::build(state, wgsl);

        ready(scope.pop()).map_or(Some(pipeline), |error| {
            tracing::warn!("the live blur is off, the GPU refused its shader: {error}");
            None
        })
    }

    fn build(state: &RenderState, wgsl: &str) -> Self {
        let device = &state.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("toolbox-frost"),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });

        let layout = bindings(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("toolbox-frost"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        let render = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("toolbox-frost"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..wgpu::SamplerDescriptor::default()
        });

        Self {
            render,
            layout,
            sampler,
        }
    }
}

fn bindings(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("toolbox-frost"),
        entries: &[
            binding(
                0,
                wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
            ),
            binding(
                1,
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            ),
            binding(
                2,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            ),
        ],
    })
}

const fn binding(index: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: index,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty,
        count: None,
    }
}

/// Native wgpu answers at once, so a scope still pending counts as no error.
fn ready(scope: impl Future<Output = Option<wgpu::Error>>) -> Option<wgpu::Error> {
    let mut scope = std::pin::pin!(scope);
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());

    match scope.as_mut().poll(&mut context) {
        std::task::Poll::Ready(error) => error,
        std::task::Poll::Pending => None,
    }
}
