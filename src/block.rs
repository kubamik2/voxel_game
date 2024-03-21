use cgmath::Point3;

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub material: Material,
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