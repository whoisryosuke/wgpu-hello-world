use std::{collections::HashMap, num::NonZeroU64, ops::Range};

use egui::epaint::Vertex;
use wgpu::util::DeviceExt;

/// Uniform buffer used when rendering.
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct UniformBuffer {
    screen_size_in_points: [f32; 2],
    // Uniform buffers need to be at least 16 bytes in WebGL.
    // See https://github.com/gfx-rs/wgpu/issues/2072
    _padding: [u32; 2],
}

struct SlicedBuffer {
    buffer: wgpu::Buffer,
    slices: Vec<Range<wgpu::BufferAddress>>,
    capacity: wgpu::BufferAddress,
}

// Utility functions
fn create_vertex_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("egui_vertex_buffer"),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        size,
        mapped_at_creation: false,
    })
}

fn create_index_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("egui_index_buffer"),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        size,
        mapped_at_creation: false,
    })
}

// The Render Pass
pub struct EguiPass {
    pipeline: wgpu::RenderPipeline,

    index_buffer: SlicedBuffer,
    vertex_buffer: SlicedBuffer,

    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    /// Map of egui texture IDs to textures and their associated bindgroups (texture view +
    /// sampler). The texture may be None if the TextureId is just a handle to a user-provided
    /// sampler.
    textures: HashMap<egui::TextureId, (Option<wgpu::Texture>, wgpu::BindGroup)>,
    next_user_texture_id: u64,
    // samplers: HashMap<TextureOptions, wgpu::Sampler>,

    // Storage for use by [`egui::PaintCallback`]'s that need to store resources such as render
    // pipelines that must have the lifetime of the renderpass.
    // pub paint_callback_resources: TypeMap,
}

impl EguiPass {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        output_color_format: wgpu::TextureFormat,
        output_depth_format: Option<wgpu::TextureFormat>,
        msaa_samples: u32,
    ) -> EguiPass {
        // Setup the shader
        // We use specific shaders for each pass to define visual effect
        // and also to have the right shader for the uniforms we pass
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Egui Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/egui.wgsl").into()),
        });

        // Setup uniforms for shader
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("egui_uniform_buffer"),
            contents: bytemuck::cast_slice(&[UniformBuffer {
                screen_size_in_points: [0.0, 0.0],
                _padding: Default::default(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("egui_uniform_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(std::mem::size_of::<UniformBuffer>() as _),
                        ty: wgpu::BufferBindingType::Uniform,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("egui_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        // Setup texture uniforms
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("egui_texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Setup render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("egui_pipeline_layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let depth_stencil = output_depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("egui_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                entry_point: "vs_main",
                module: &shader_module,
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 5 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    // 0: vec2 position
                    // 1: vec2 texture coordinates
                    // 2: uint color
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint32],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                unclipped_depth: false,
                conservative: false,
                cull_mode: None,
                front_face: wgpu::FrontFace::default(),
                polygon_mode: wgpu::PolygonMode::default(),
                strip_index_format: None,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                alpha_to_coverage_enabled: false,
                count: msaa_samples,
                mask: !0,
            },

            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: if output_color_format.describe().srgb {
                    println!("Detected a linear (sRGBA aware) framebuffer {:?}. egui prefers Rgba8Unorm or Bgra8Unorm", output_color_format);
                    "fs_main_linear_framebuffer"
                } else {
                    "fs_main_gamma_framebuffer" // this is what we prefer
                },
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_color_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        const VERTEX_BUFFER_START_CAPACITY: wgpu::BufferAddress =
            (std::mem::size_of::<Vertex>() * 1024) as _;
        const INDEX_BUFFER_START_CAPACITY: wgpu::BufferAddress =
            (std::mem::size_of::<u32>() * 1024 * 3) as _;

        Self {
            pipeline,
            vertex_buffer: SlicedBuffer {
                buffer: create_vertex_buffer(device, VERTEX_BUFFER_START_CAPACITY),
                slices: Vec::with_capacity(64),
                capacity: VERTEX_BUFFER_START_CAPACITY,
            },
            index_buffer: SlicedBuffer {
                buffer: create_index_buffer(device, INDEX_BUFFER_START_CAPACITY),
                slices: Vec::with_capacity(64),
                capacity: INDEX_BUFFER_START_CAPACITY,
            },
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout,
            textures: HashMap::new(),
            next_user_texture_id: 0,
            // samplers: HashMap::new(),
            // paint_callback_resources: TypeMap::default(),
        }
    }
}
