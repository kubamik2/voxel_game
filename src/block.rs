use cgmath::{Point3};

pub struct Block {
    pub position: Point3<f32>,
    pub material: Material
}

pub enum Material {
    Cobblestone
}