use crate::{
    model::{self, Material, ModelVertex},
    resources::load_texture,
    texture::Texture,
    Vertex,
};
use std::ops::Range;
use wgpu::util::DeviceExt;
pub mod cube;

/// General methods shared with all primitives
// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct PrimitiveVertex {
//     pub position: [f32; 3],
//     pub color: [f32; 3],
// }

// impl Vertex for PrimitiveVertex {
//     fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         use std::mem;
//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<PrimitiveVertex>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &[
//                 // Position
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 0,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 // Color
//                 wgpu::VertexAttribute {
//                     offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
//                     shadef_location: 1,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//             ],
//         }
//     }
// }
pub struct PrimitiveMesh {
    pub model: model::Model,
}

impl PrimitiveMesh {
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        vertices: &[ModelVertex],
        indices: &Vec<u32>,
    ) -> Self {
        let primitive_type = "Cube";
        // let num_vertices = vertices.len() as u32;
        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Primitive Vertex Buffer"),
        //     contents: bytemuck::cast_slice(vertices),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        // Setup materials
        // We can't have empty material (since shader relies o n bind group)
        // And it doesn't accept Option/None, so we give it a placeholder image
        let mut materials = Vec::new();
        let diffuse_texture = load_texture(&"banana.png", device, queue)
            .await
            .expect("Couldn't load placeholder texture for primitive");
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        });

        materials.push(model::Material {
            name: primitive_type.to_string(),
            diffuse_texture,
            bind_group,
        });

        let mut meshes = Vec::new();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", primitive_type)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", primitive_type)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        meshes.push(model::Mesh {
            name: primitive_type.to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
        });

        let model = model::Model { meshes, materials };

        Self { model }
    }
}
