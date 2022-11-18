use crate::{instance::Instance, model, pass::phong::Locals};

// This represents a 3D model in a scene.
// It contains the 3D model, instance data, and a parent ID (TBD)
pub struct Node {
    // ID of parent Node
    pub parent: u32,
    // local: Matrix?
    // Local position of model (for relative calculations)
    pub locals: Locals,
    // The vertex buffers and texture data
    pub model: model::Model,
    // An array of positional data for each instance (can just pass 1 instance)
    pub instances: Vec<Instance>,
}
