use crate::{
    model::{self, Material, ModelVertex},
    resources::load_texture,
    texture::Texture,
    Vertex,
};
use std::ops::Range;
use wgpu::util::DeviceExt;
pub mod cube;
pub mod plane;
pub struct PrimitiveMesh {
    pub model: model::Model,
}

impl PrimitiveMesh {
    pub async fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[ModelVertex],
        indices: &Vec<u32>,
    ) -> Self {
        let primitive_type = "Cube";

        println!("[PRIMITIVE] Creating cube materials");
        // Setup materials
        // We can't have empty material (since shader relies o n bind group)
        // And it doesn't accept Option/None, so we give it a placeholder image
        let mut materials = Vec::new();
        let diffuse_texture = load_texture(&"default_texture.png", device, queue)
            .await
            .expect("Couldn't load placeholder texture for primitive");

        materials.push(model::Material {
            name: primitive_type.to_string(),
            diffuse_texture,
        });

        println!("[PRIMITIVE] Creating cube mesh buffers");
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
