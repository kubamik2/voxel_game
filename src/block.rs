use cgmath::Point3;

use crate::block_vertex::{BlockVertex, Face, RawBlockVertex};

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub material: Material,
    pub adjacent_blocks_bitmap: AdjacentBlockBitmap
}
impl Block {
    pub const FACE_VERTICES: [BlockVertex; 4] = [
        BlockVertex { position: Point3 { x: 0, y: 0, z: 0 }, chunk_index: 0 },
        BlockVertex { position: Point3 { x: 0, y: 0, z: 1 }, chunk_index: 0 },
        BlockVertex { position: Point3 { x: 0, y: 1, z: 0 }, chunk_index: 0 },
        BlockVertex { position: Point3 { x: 0, y: 1, z: 1 }, chunk_index: 0 },
    ];

    pub fn raw_vertex_face_data() -> [RawBlockVertex; 4] {
        [
            Self::FACE_VERTICES[0].to_raw(),
            Self::FACE_VERTICES[1].to_raw(),
            Self::FACE_VERTICES[2].to_raw(),
            Self::FACE_VERTICES[3].to_raw(),
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AdjacentBlockBitmap(pub u8);

impl AdjacentBlockBitmap {
    pub fn set_face(&mut self, index: usize, value: bool) {
        let shifted_value = (value as u8) << index;
        let mask = !(1 << index);
        self.0 = (mask & self.0) | shifted_value;
    }

    pub fn face_visible(&self, index: usize) -> bool {
        (self.0 & (1 << index)) > 0
    }
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Air,
    Cobblestone,
    Dirt,
    Grass,
}

impl Material {
    pub fn text_coords(&self) -> [u16; 6] {
        match self {
            Self::Air => [0, 1, 2, 3, 4, 5],
            Self::Cobblestone => [6, 7, 8, 9, 10, 11],
            Self::Dirt => [12, 13, 14, 15, 16, 17],
            Self::Grass => [18, 19, 20, 21, 22, 23],
        }
    }
}