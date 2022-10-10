use crate::Vertex;
use std::ops::Range;
use wgpu::util::DeviceExt;
pub mod cube;

/// General methods shared with all primitives
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PrimitiveVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex for PrimitiveVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<PrimitiveVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
pub struct PrimitiveMesh {
    pub vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl PrimitiveMesh {
    pub fn new(device: &wgpu::Device, vertices: &[PrimitiveVertex]) -> Self {
        let num_vertices = vertices.len() as u32;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Primitive Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            num_vertices,
            vertex_buffer,
        }
    }
}
pub trait DrawPrimitive<'a> {
    fn draw_primitive(
        &mut self,
        mesh: &'a PrimitiveMesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        play_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_primitive_instanced(
        &mut self,
        mesh: &'a PrimitiveMesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        play_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawPrimitive<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_primitive(
        &mut self,
        mesh: &'b PrimitiveMesh,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        play_bind_group: &'a wgpu::BindGroup,
    ) {
        self.draw_primitive_instanced(
            mesh,
            0..1,
            camera_bind_group,
            light_bind_group,
            play_bind_group,
        );
    }

    fn draw_primitive_instanced(
        &mut self,
        mesh: &'b PrimitiveMesh,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        play_bind_group: &'a wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        // self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        self.set_bind_group(3, play_bind_group, &[]);
        // self.draw_indexed(0..mesh.num_elements, 0, instances);
        // self.draw_indexed(0..1, 0, instances);
        self.draw(0..mesh.num_vertices, instances);

        //         render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        //         render_pass.draw(0..self.num_vertices, 0..1);
    }
}
