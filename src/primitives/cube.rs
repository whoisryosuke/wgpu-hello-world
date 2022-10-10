use super::PrimitiveVertex;

pub const CUBE_VERTICES: &[PrimitiveVertex] = &[
    PrimitiveVertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    PrimitiveVertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    PrimitiveVertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];
