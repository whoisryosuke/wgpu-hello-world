use crate::model::ModelVertex;

pub fn plane_vertices(scale: f32) -> Vec<ModelVertex> {
    vec![
        // Front face
        ModelVertex {
            position: [-scale, -scale, scale],
            normal: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
        },
        ModelVertex {
            position: [scale, -scale, scale],
            normal: [0.0, 0.0, -1.0],
            tex_coords: [1.0, 0.0],
        },
        ModelVertex {
            position: [scale, scale, scale],
            normal: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
        },
        ModelVertex {
            position: [-scale, scale, scale],
            normal: [-1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
        },
    ]
}

pub fn plane_indices() -> Vec<u32> {
    vec![
        0, 1, 2, 0, 2, 3, // front
    ]
}
