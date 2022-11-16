use wgpu::{Device, Queue, Surface};

use crate::{model::Model, node::Node};

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
