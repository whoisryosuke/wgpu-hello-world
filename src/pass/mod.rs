use wgpu::{Device, Queue, Surface};

use crate::{model::Model, node::Node};

pub mod egui;
pub mod phong;

pub trait Pass {
    fn draw(
        &mut self,
        surface: &Surface,
        device: &Device,
        queue: &Queue,
        nodes: &Vec<Node>,
    ) -> Result<(), wgpu::SurfaceError>;
}

/// Uniform buffer pool
/// Used by render passes to keep track of each objects local uniforms
/// and provides a way to update uniforms to render pipeline
pub struct UniformPool {
    label: &'static str,
    pub buffers: Vec<wgpu::Buffer>,
    size: u64,
}

impl UniformPool {
    pub fn new(label: &'static str, size: u64) -> Self {
        Self {
            label,
            buffers: Vec::new(),
            size,
        }
    }

    pub fn alloc_buffers(&mut self, count: usize, device: &Device) {
        // We reset the buffers each time we allocate
        // TODO: Ideally we should keep track of the object it belongs to,
        // so we can add/remove objects (and their uniform buffers) dynamically
        self.buffers = Vec::new();

        for _ in 0..count {
            let local_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&self.label),
                size: self.size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffers.push(local_uniform_buffer);
        }
    }

    pub fn update_uniform<T: bytemuck::Pod>(&self, index: usize, data: T, queue: &Queue) {
        if &self.buffers.len() > &0 {
            queue.write_buffer(&self.buffers[index], 0, bytemuck::cast_slice(&[data]));
        }
    }
}
