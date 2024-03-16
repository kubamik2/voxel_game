
use cgmath::Point3;
use crate::vertex::{Vertex, VertexPacked};

pub struct Block {
    pub position: Point3<f32>,
    pub material: Material,
    pub bitmap: Bitmap
}

impl Block {
    pub const FACE_VERTICES: [[Vertex; 4]; 6] = [
        [
            Vertex { position: [1.0, 0.0, 0.0], tex_coords: [0, 16], tile: [1, 1] }, // 1
            Vertex { position: [1.0, 0.0, -1.0], tex_coords: [16, 16], tile: [1, 1] },
            Vertex { position: [1.0, 1.0, 0.0], tex_coords: [0, 0], tile: [1, 1] },
            Vertex { position: [1.0, 1.0, -1.0], tex_coords: [16, 0], tile: [1, 1] },
        ],
        [
            Vertex { position: [0.0, 0.0, -1.0], tex_coords: [0, 16], tile: [1, 1] }, // 3
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [16, 16], tile: [1, 1] },
            Vertex { position: [0.0, 1.0, -1.0], tex_coords: [0, 0], tile: [1, 1] },
            Vertex { position: [0.0, 1.0, 0.0], tex_coords: [16, 0], tile: [1, 1] },
        ],
        [
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0, 16], tile: [1, 1] }, // 0
            Vertex { position: [1.0, 0.0, 0.0], tex_coords: [16, 16], tile: [1, 1] },
            Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0, 0], tile: [1, 1] },
            Vertex { position: [1.0, 1.0, 0.0], tex_coords: [16, 0], tile: [1, 1] },
        ],
        [
            Vertex { position: [1.0, 0.0, -1.0], tex_coords: [0, 16], tile: [1, 1] }, // 2
            Vertex { position: [0.0, 0.0, -1.0], tex_coords: [16, 16], tile: [1, 1] },
            Vertex { position: [1.0, 1.0, -1.0], tex_coords: [0, 0], tile: [1, 1] },
            Vertex { position: [0.0, 1.0, -1.0], tex_coords: [16, 0], tile: [1, 1] },
        ],
        [
            Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0, 16], tile: [1, 1] }, // 5
            Vertex { position: [1.0, 1.0, 0.0], tex_coords: [16, 16], tile: [1, 1] },
            Vertex { position: [0.0, 1.0, -1.0], tex_coords: [0, 0], tile: [1, 1] },
            Vertex { position: [1.0, 1.0, -1.0], tex_coords: [16, 0], tile: [1, 1] },
        ],
        [
            Vertex { position: [0.0, 0.0, -1.0], tex_coords: [0, 16], tile: [1, 1] }, // 4
            Vertex { position: [1.0, 0.0, -1.0], tex_coords: [16, 16], tile: [1, 1] },
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0, 0], tile: [1, 1] },
            Vertex { position: [1.0, 0.0, 0.0], tex_coords: [16, 0], tile: [1, 1] },
        ],
    ];

    pub const FACE_INDICES: [u32; 6] = [
        0, 1, 2,
        1, 3, 2,
    ];

    #[inline]
    pub fn positive_x_occupied(&self) -> bool {
        self.bitmap.get(0)
    }

    #[inline]
    pub fn negative_x_occupied(&self) -> bool {
        self.bitmap.get(1)
    }

    #[inline]
    pub fn positive_z_occupied(&self) -> bool {
        self.bitmap.get(2)
    }

    #[inline]
    pub fn negative_z_occupied(&self) -> bool {
        self.bitmap.get(3)
    }

    #[inline]
    pub fn positive_y_occupied(&self) -> bool {
        self.bitmap.get(4)
    }

    #[inline]
    pub fn negative_y_occupied(&self) -> bool {
        self.bitmap.get(5)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Cobblestone,
    Air,
    Dirt,
    Grass,
}

impl Material {
    pub fn texture_offsets(&self) -> [cgmath::Vector2<u8>; 6]{
        match *self {
            Self::Air => [(0, 0).into(); 6],
            Self::Cobblestone => [(0, 0).into(); 6],
            Self::Dirt => [(16, 0).into(); 6],
            Self::Grass => [(2 * 16, 0).into(), (2 * 16, 0).into(), (2 * 16, 0).into(), (2 * 16, 0).into(), (3 * 16, 0).into(), (16, 0).into()]
        }
    }
}

pub struct Bitmap(pub u8);

impl Bitmap {
    #[inline]
    pub fn get(&self, index: usize) -> bool {
        (self.0 & (1 << index)) > 0
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        let mask = !(1 << index);
        self.0 = self.0 & mask | ((value as u8) << index);
    }
}