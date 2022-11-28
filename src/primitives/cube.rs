use crate::model::ModelVertex;

pub fn cube_vertices(scale: f32) -> Vec<ModelVertex> {
    vec![
        // Front face
        ModelVertex {
            position: [-scale, -scale, scale],
            normal: [0.0, 0.0, scale],
            tex_coords: [0.0, 0.0],
        },
        ModelVertex {
            position: [scale, -scale, scale],
            normal: [0.0, 0.0, -scale],
            tex_coords: [1.0, 0.0],
        },
        ModelVertex {
            position: [scale, scale, scale],
            normal: [scale, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
        },
        ModelVertex {
            position: [-scale, scale, scale],
            normal: [-scale, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
        },
        // Back face
        ModelVertex {
            position: [-scale, -scale, -scale],
            normal: [0.0, scale, 0.0],
            tex_coords: [0.0, 0.0],
        },
        ModelVertex {
            position: [-scale, scale, -scale],
            normal: [0.0, -scale, 0.0],
            tex_coords: [1.0, 0.0],
        },
        ModelVertex {
            position: [scale, scale, -scale],
            normal: [0.0, 0.0, scale],
            tex_coords: [1.0, 1.0],
        },
        ModelVertex {
            position: [scale, -scale, -scale],
            normal: [0.0, 0.0, -scale],
            tex_coords: [0.0, 1.0],
        },
        // Top face
        ModelVertex {
            position: [-scale, scale, -scale],
            normal: [scale, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
        },
        ModelVertex {
            position: [-scale, scale, scale],
            normal: [-scale, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
        },
        ModelVertex {
            position: [scale, scale, scale],
            normal: [0.0, scale, 0.0],
            tex_coords: [1.0, 1.0],
        },
        ModelVertex {
            position: [scale, scale, -scale],
            normal: [0.0, -scale, 0.0],
            tex_coords: [0.0, 1.0],
        },
        // Bottom face
        ModelVertex {
            position: [-scale, -scale, -scale],
            normal: [0.0, 0.0, scale],

            tex_coords: [0.0, 0.0],
        },
        ModelVertex {
            position: [scale, -scale, -scale],
            normal: [0.0, 0.0, -scale],
            tex_coords: [1.0, 0.0],
        },
        ModelVertex {
            position: [scale, -scale, scale],
            normal: [scale, 0.0, 0.0],

            tex_coords: [1.0, 1.0],
        },
        ModelVertex {
            position: [-scale, -scale, scale],
            normal: [-scale, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
        },
        // Right face
        ModelVertex {
            position: [scale, -scale, -scale],
            normal: [0.0, scale, 0.0],
            tex_coords: [0.0, 0.0],
        },
        ModelVertex {
            position: [scale, scale, -scale],
            normal: [0.0, -scale, 0.0],
            tex_coords: [1.0, 0.0],
        },
        ModelVertex {
            position: [scale, scale, scale],
            normal: [0.0, 0.0, scale],
            tex_coords: [1.0, 1.0],
        },
        ModelVertex {
            position: [scale, -scale, scale],
            normal: [0.0, 0.0, -scale],
            tex_coords: [0.0, 1.0],
        },
        // Left face
        ModelVertex {
            position: [-scale, -scale, -scale],
            normal: [scale, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
        },
        ModelVertex {
            position: [-scale, -scale, scale],
            normal: [-scale, 0.0, 0.0],
            tex_coords: [1.0, 0.0],
        },
        ModelVertex {
            position: [-scale, scale, scale],
            normal: [0.0, scale, 0.0],
            tex_coords: [1.0, 1.0],
        },
        ModelVertex {
            position: [-scale, scale, -scale],
            normal: [0.0, -scale, 0.0],
            tex_coords: [0.0, 1.0],
        },
    ]
}

pub fn cube_indices() -> Vec<u32> {
    vec![
        0, 1, 2, 0, 2, 3, // front
        4, 5, 6, 4, 6, 7, // back
        8, 9, 10, 8, 10, 11, // top
        12, 13, 14, 12, 14, 15, // bottom
        16, 17, 18, 16, 18, 19, // right
        20, 21, 22, 20, 22, 23, // left
    ]
}
