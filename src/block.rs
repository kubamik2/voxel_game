use cgmath::Point3;

use crate::block_vertex::*;

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub material: Material,
}

impl Block {
    pub const FACE_VERTICES: [[BlockVertex; 4]; 6] = [
        [
            BlockVertex { position: Point3::new(1, 0, 1), face: Face::PositiveX },
            BlockVertex { position: Point3::new(1, 0, 0), face: Face::PositiveX },
            BlockVertex { position: Point3::new(1, 1, 1), face: Face::PositiveX },
            BlockVertex { position: Point3::new(1, 1, 0), face: Face::PositiveX },
        ],
        [
            BlockVertex { position: Point3::new(0, 0, 0), face: Face::NegativeX },
            BlockVertex { position: Point3::new(0, 0, 1), face: Face::NegativeX },
            BlockVertex { position: Point3::new(0, 1, 0), face: Face::NegativeX },
            BlockVertex { position: Point3::new(0, 1, 1), face: Face::NegativeX },
        ],
        [
            BlockVertex { position: Point3::new(0, 0, 1), face: Face::PositiveZ },
            BlockVertex { position: Point3::new(1, 0, 1), face: Face::PositiveZ },
            BlockVertex { position: Point3::new(0, 1, 1), face: Face::PositiveZ },
            BlockVertex { position: Point3::new(1, 1, 1), face: Face::PositiveZ },
        ],
        [
            BlockVertex { position: Point3::new(1, 0, 0), face: Face::NegativeZ },
            BlockVertex { position: Point3::new(0, 0, 0), face: Face::NegativeZ },
            BlockVertex { position: Point3::new(1, 1, 0), face: Face::NegativeZ },
            BlockVertex { position: Point3::new(0, 1, 0), face: Face::NegativeZ },
        ],
        [
            BlockVertex { position: Point3::new(0, 1, 0), face: Face::PositiveY },
            BlockVertex { position: Point3::new(0, 1, 1), face: Face::PositiveY },
            BlockVertex { position: Point3::new(1, 1, 0), face: Face::PositiveY },
            BlockVertex { position: Point3::new(1, 1, 1), face: Face::PositiveY },
        ],
        [
            BlockVertex { position: Point3::new(1, 0, 0), face: Face::NegativeY },
            BlockVertex { position: Point3::new(1, 0, 1), face: Face::NegativeY },
            BlockVertex { position: Point3::new(0, 0, 0), face: Face::NegativeY },
            BlockVertex { position: Point3::new(0, 0, 1), face: Face::NegativeY },
        ],
    ];

    pub const FACE_INDICES: [u32; 6] = [
        2, 0, 1,
        3, 2, 1
    ];
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Air,
    Cobblestone,
    Dirt,
    Grass,
}