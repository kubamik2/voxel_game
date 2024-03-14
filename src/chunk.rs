pub const CHUNK_SIZE: usize = 16;
use std::ops::{Index, IndexMut};

use crate::block::{Block, Material};

pub struct Chunk {
    pub blocks: Vec<Block>
}

impl Index<(usize, usize, usize)> for Chunk {
    type Output = Block;
    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        &self.blocks[index.0 + index.1 * 256 + index.2 * CHUNK_SIZE]
    }
}

impl IndexMut<(usize, usize, usize)> for Chunk {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        &mut self.blocks[index.0 + index.1 * 256 + index.2 * CHUNK_SIZE]
    }
}

impl Chunk {
    pub fn bake(&mut self) {
        for y in 0..256 {
            for z in 0..16 {
                for x in 0..16 {
                    if x + 1 < CHUNK_SIZE {
                        let v = self[(x + 1, y, z)].material != Material::Air;
                        self[(x, y, z)].bitmap.set(0, v);
                    }
                    
                    if x > 0 {
                        let v = self[(x - 1, y, z)].material != Material::Air;
                        self[(x, y, z)].bitmap.set(1, v);
                    }
                    
                    if z + 1 < CHUNK_SIZE {
                        let v = self[(x, y, z + 1)].material != Material::Air;
                        self[(x, y, z)].bitmap.set(2, v);
                    }

                    if z > 0 {
                        let v = self[(x, y, z - 1)].material != Material::Air;
                        self[(x, y, z)].bitmap.set(3, v);
                    }

                    if y + 1 < 256 {
                        let v = self[(x, y + 1, z)].material != Material::Air;
                        self[(x, y, z)].bitmap.set(4, v);
                    }

                    if y > 0 {
                        let v = self[(x, y - 1, z)].material != Material::Air;
                        self[(x, y, z)].bitmap.set(5, v);
                    }
                }
            }
        }
    }
}