use crate::model::ModelVertex;

pub const CUBE_VERTICES: &[ModelVertex] = &[
    ModelVertex {
        position: [0.0, 0.5, 0.0],
        normal: [0.0, 0.5, 0.0],
        tex_coords: [0.0, 0.5],
    },
    ModelVertex {
        position: [-0.5, -0.5, 0.0],
        normal: [-0.5, -0.5, 0.0],
        tex_coords: [0.5, 0.5],
    },
    ModelVertex {
        position: [0.5, -0.5, 0.0],
        normal: [0.5, -0.5, 0.0],
        tex_coords: [0.5, 0.5],
    },
];
// pub const CUBE_INDICES: &Vec<u32> = &vec![0, 1, 2];
pub fn cube_indices() -> Vec<u32> {
    vec![0, 1, 2]
}
// pub fn cube_indices() -> Vec<u32> {
//     vec![2,3,4,5,8,7,5,6,2,2,6,7,7,8,4,5,1,4,1,2,4,6,5,7,1,5,2,3,2,7,3,7,4,8,5,4]
// }
// pub const CUBE_INDICES: &Vec<u32> = &vec![0, 1, 2];

// position: [0.500000, -0.500000, -0.500000],
// normal: [0.000000, -1.000000, 0.000000],
// position: [0.500000, -0.500000, 0.500000],
// normal: [0.000000, 1.000000, 0.000000],
// position: [-0.500000, -0.500000, 0.500000],
// normal: [1.000000, 0.000000, 0.000001],
// position: [-0.500000, -0.500000, -0.500000],
// normal: [-0.000000, -0.000000, 1.000000],
// position: [0.500000, 0.500000, -0.500000],
// normal: [-1.000000, -0.000000, -0.000000],
// position: [0.500000, 0.500000, 0.500000],
// normal: [0.000000, 0.000000, -1.000000],
// position: [-0.500000, 0.500000, 0.500000],
// normal: [1.000000, -0.000000, 0.000000],
// position: [-0.500000, 0.500000, -0.500000],
