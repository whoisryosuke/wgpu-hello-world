use std::{collections::HashMap, iter, mem};

use cgmath::{InnerSpace, Rotation3, Zero};
use wgpu::{util::DeviceExt, BindGroupLayout, Device, Queue, Surface};

use crate::{
    camera::{Camera, CameraUniform},
    context::create_render_pipeline,
    instance::{Instance, InstanceRaw},
    model::{self, DrawLight, DrawModel, Model, Vertex},
    node::Node,
    texture,
};

use super::{Pass, UniformPool};

// Global uniform data
// aka camera position and ambient light color
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    ambient: [f32; 4],
}

// Local uniform data
// aka the individual model's data
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Locals {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub normal: [f32; 4],
    pub lights: [f32; 4],
}

// Uniform for light data (position + color)
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    pub color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

pub struct PhongConfig {
    pub max_lights: usize,
    pub ambient: [u32; 4],
    pub wireframe: bool,
}

pub struct PhongPass {
    // Uniforms
    pub global_bind_group_layout: BindGroupLayout,
    pub global_uniform_buffer: wgpu::Buffer,
    pub global_bind_group: wgpu::BindGroup,
    pub local_bind_group_layout: BindGroupLayout,
    // pub local_uniform_buffer: wgpu::Buffer,
    local_bind_groups: HashMap<usize, wgpu::BindGroup>,
    pub uniform_pool: UniformPool,
    // Textures
    pub depth_texture: texture::Texture,
    // Render pipeline
    pub render_pipeline: wgpu::RenderPipeline,
    // Lighting
    pub light_uniform: LightUniform,
    pub light_buffer: wgpu::Buffer,
    // pub light_bind_group: wgpu::BindGroup,
    pub light_render_pipeline: wgpu::RenderPipeline,
    // Camera
    pub camera_uniform: CameraUniform,
    // Instances
    instance_buffers: HashMap<usize, wgpu::Buffer>,
}

impl PhongPass {
    pub fn new(
        phong_config: &PhongConfig,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        camera: &Camera,
    ) -> PhongPass {
        // Setup the shader
        // We use specific shaders for each pass to define visual effect
        // and also to have the right shader for the uniforms we pass
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Normal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        // Setup global uniforms
        // Global bind group layout
        let light_size = mem::size_of::<LightUniform>() as wgpu::BufferAddress;
        let global_size = mem::size_of::<Globals>() as wgpu::BufferAddress;
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("[Phong] Globals"),
                entries: &[
                    // Global uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(global_size),
                        },
                        count: None,
                    },
                    // Lights
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(light_size),
                        },
                        count: None,
                    },
                    // Sampler for textures
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Global uniform buffer
        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("[Phong] Globals"),
            size: global_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Create light uniforms and setup buffer for them
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("[Phong] Lights"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // We also need a sampler for our textures
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("[Phong] sampler"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        // Combine the global uniform, the lights, and the texture sampler into one bind group
        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("[Phong] Globals"),
            layout: &global_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Setup local uniforms
        // Local bind group layout
        let local_size = mem::size_of::<Locals>() as wgpu::BufferAddress;
        let local_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("[Phong] Locals"),
                entries: &[
                    // Local uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(local_size),
                        },
                        count: None,
                    },
                    // Mesh texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Setup the render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("[Phong] Pipeline"),
            bind_group_layouts: &[&global_bind_group_layout, &local_bind_group_layout],
            push_constant_ranges: &[],
        });
        let vertex_buffers = [model::ModelVertex::desc(), InstanceRaw::desc()];
        let depth_stencil = Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        });

        // Enable/disable wireframe mode
        let topology = if phong_config.wireframe {
            wgpu::PrimitiveTopology::LineList
        } else {
            wgpu::PrimitiveTopology::TriangleList
        };

        let primitive = wgpu::PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
            topology,
            ..Default::default()
        };
        let multisample = wgpu::MultisampleState {
            ..Default::default()
        };
        let color_format = texture::Texture::DEPTH_FORMAT;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("[Phong] Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            primitive,
            depth_stencil: depth_stencil.clone(),
            multisample,
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        // Create depth texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        // Setup camera uniform
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let light_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Light Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../light.wgsl").into()),
        });

        let light_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("[Phong] Light Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &light_shader,
                    entry_point: "vs_main",
                    buffers: &[model::ModelVertex::desc()],
                },
                primitive,
                depth_stencil,
                multisample,
                fragment: Some(wgpu::FragmentState {
                    module: &light_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        // Create instance buffer
        let instance_buffers = HashMap::new();

        let uniform_pool = UniformPool::new("[Phong] Locals", local_size);

        PhongPass {
            global_bind_group_layout,
            global_uniform_buffer,
            global_bind_group,
            local_bind_group_layout,
            local_bind_groups: Default::default(),
            uniform_pool,
            depth_texture,
            render_pipeline,
            camera_uniform,
            light_uniform,
            light_buffer,
            light_render_pipeline,
            instance_buffers,
        }
    }
}

impl Pass for PhongPass {
    fn draw(
        &mut self,
        surface: &Surface,
        device: &Device,
        queue: &Queue,
        nodes: &Vec<Node>,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Setup the render pass
        // see: clear color, depth stencil
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Set the clear color during redraw
                        // This is basically a background color applied if an object isn't taking up space
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                // Create a depth stencil buffer using the depth texture
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // Allocate buffers for local uniforms
            if (self.uniform_pool.buffers.len() < nodes.len()) {
                self.uniform_pool.alloc_buffers(nodes.len(), &device);
            }

            // Loop over the nodes/models in a scene and setup the specific models
            // local uniform bind group and instance buffers to send to shader
            // This is separate loop from the render because of Rust ownership
            // (can prob wrap in block instead to limit mutable use)
            let mut model_index = 0;
            for node in nodes {
                let local_buffer = &self.uniform_pool.buffers[model_index];

                // We create a bind group for each model's local uniform data
                // and store it in a hash map to look up later
                self.local_bind_groups
                    .entry(model_index)
                    .or_insert_with(|| {
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("[Phong] Locals"),
                            layout: &self.local_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: local_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(
                                        &node.model.materials[0].diffuse_texture.view,
                                    ),
                                },
                            ],
                        })
                    });

                // Setup instance buffer for the model
                // similar process as above using HashMap
                self.instance_buffers.entry(model_index).or_insert_with(|| {
                    // We condense the matrix properties into a flat array (aka "raw data")
                    // (which is how buffers work - so we can "stride" over chunks)
                    let instance_data = node
                        .instances
                        .iter()
                        .map(Instance::to_raw)
                        .collect::<Vec<_>>();
                    // Create the instance buffer with our data
                    let instance_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Instance Buffer"),
                            contents: bytemuck::cast_slice(&instance_data),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                    instance_buffer
                });

                model_index += 1;
            }

            // Setup lighting pipeline
            render_pass.set_pipeline(&self.light_render_pipeline);
            // Draw/calculate the lighting on models
            render_pass.draw_light_model(
                &nodes[1].model,
                &self.global_bind_group,
                &self
                    .local_bind_groups
                    .get(&1)
                    .expect("No local bind group found for lighting"),
            );

            // Setup render pipeline
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.global_bind_group, &[]);

            // Render/draw all nodes/models
            // We reset index here to use again
            model_index = 0;
            for node in nodes {
                // Set the instance buffer unique to the model
                render_pass.set_vertex_buffer(1, self.instance_buffers[&model_index].slice(..));

                // Draw all the model instances
                render_pass.draw_model_instanced(
                    &node.model,
                    0..*&node.instances.len() as u32,
                    &self.local_bind_groups[&model_index],
                );

                model_index += 1;
            }
        }

        queue.submit(Some(encoder.finish()));
        output.present();

        // Since the WGPU breaks return with a Result and error
        // we need to return an `Ok` enum
        Ok(())
    }
}
