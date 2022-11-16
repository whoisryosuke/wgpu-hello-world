use crate::{instance::Instance, model};

pub struct Node {
    pub parent: u32,
    // local: Matrix?
    pub model: model::Model,
    pub instances: Vec<Instance>,
}
