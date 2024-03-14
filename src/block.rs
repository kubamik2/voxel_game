
use cgmath::{Point3};
use crate::vertex::Vertex;

pub struct Block {
    pub position: Point3<f32>,
    pub material: Material,
    pub bitmap: Bitmap
}

impl Block {
    pub const VERTICES: &'static [Vertex] = &[
        Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0625] }, // 0
        Vertex { position: [1.0, 0.0, 0.0], tex_coords: [0.0625, 0.0625] },
        Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [1.0, 1.0, 0.0], tex_coords: [0.0625, 0.0] },

        // Vertex { position: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0625] }, // 4
        // Vertex { position: [1.0, 0.0, -1.0], tex_coords: [0.0625, 0.0625] },
        // Vertex { position: [1.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },
        // Vertex { position: [1.0, 1.0, -1.0], tex_coords: [0.0625, 0.0] },

        // Vertex { position: [1.0, 0.0, -1.0], tex_coords: [0.0, 0.0625] }, // 8
        // Vertex { position: [0.0, 0.0, -1.0], tex_coords: [0.0625, 0.0625] },
        // Vertex { position: [1.0, 1.0, -1.0], tex_coords: [0.0, 0.0] },
        // Vertex { position: [0.0, 1.0, -1.0], tex_coords: [0.0625, 0.0] },

        // Vertex { position: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0625] }, // 12
        // Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0625, 0.0625] },
        // Vertex { position: [0.0, 1.0, -1.0], tex_coords: [0.0, 0.0] },
        // Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0.0625, 0.0] },

        // Vertex { position: [0.0, 0.0, -1.0], tex_coords: [0.0, 0.0625] }, // 16
        // Vertex { position: [1.0, 0.0, -1.0], tex_coords: [0.0625, 0.0625] },
        // Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
        // Vertex { position: [1.0, 0.0, 0.0], tex_coords: [0.0625, 0.0] },

        // Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0625] }, // 20
        // Vertex { position: [1.0, 1.0, 0.0], tex_coords: [0.0625, 0.0625] },
        // Vertex { position: [0.0, 1.0, -1.0], tex_coords: [0.0, 0.0] },
        // Vertex { position: [1.0, 1.0, -1.0], tex_coords: [0.0625, 0.0] },
    ];

    pub const INDICES: &'static [u16] = &[
        0, 1, 2,
        1, 3, 2,

        // 4, 5, 6,
        // 5, 7, 6,

        // 8, 9, 10,
        // 9, 11, 10,

        // 12, 13, 14,
        // 13, 15, 14,

        // 16, 17, 18,
        // 17, 19, 18,

        // 20, 21, 22,
        // 21, 23, 22,
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

    // #[inline]
    // pub fn push_vertices(&self, vertices_buffer: &mut Vec<Vertex>) {
    //     if !self.positive_x_occupied() {
    //         vertices_buffer.extend([
    //             Vertex { position: [1.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
    //             Vertex { position: [1.0, 0.0, 1.0], tex_coords: [0.0, 0.0] },
    //             Vertex { position: [1.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },

    //             Vertex { position: [1.0, 0.0, 1.0], tex_coords: [0.0, 0.0] },
    //             Vertex { position: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
    //             Vertex { position: [1.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },
    //         ]);
    //     }

    //     if !self.negative_x_occupied() {
    //         vertices_buffer.extend([
    //             Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
    //             Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },
    //             Vertex { position: [0.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },

    //             Vertex { position: [0.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
    //             Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
    //             Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
    //         ]);
    //     }
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Cobblestone,
    Air,
    Dirt,
    Grass,
}

impl Material {
    pub fn texture_offsets(&self) -> [cgmath::Vector2<f32>; 6]{
        match *self {
            Self::Air => [(0.0, 0.0).into(); 6],
            Self::Cobblestone => [(0.0, 0.0).into(); 6],
            Self::Dirt => [(0.0625, 0.0).into(); 6],
            Self::Grass => [(2.0 * 0.0625, 0.0).into(), (2.0 * 0.0625, 0.0).into(), (2.0 * 0.0625, 0.0).into(), (2.0 * 0.0625, 0.0).into(), (3.0 * 0.0625, 0.0).into(), (0.0625, 0.0).into()]
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