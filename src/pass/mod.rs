use wgpu::{Device, Queue, Surface};

use crate::model::Model;

pub mod phong;

pub trait Pass {
    fn draw(
        &mut self,
        surface: &Surface,
        device: &Device,
        queue: &Queue,
        obj_model: &Model,
    ) -> Result<(), wgpu::SurfaceError>;
}
