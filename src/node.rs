use crate::{instance::Instance, model};

// This represents a 3D model in a scene.
// It contains the 3D model, instance data, and a parent ID (TBD)
pub struct Node {
    pub parent: u32,
    // local: Matrix?
    pub model: model::Model,
    pub instances: Vec<Instance>,
}
