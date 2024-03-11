pub const CHUNK_SIZE: usize = 16;
use std::ops::{Index, IndexMut};

use crate::block::Block;

pub struct Chunk {
    blocks: [Block; 16 * 16]
}

impl Index<(usize, usize)> for Chunk {
    type Output = Block;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        if index.0 >= CHUNK_SIZE { panic!("Out of bound access") }
        &self.blocks[index.0 + index.1 * CHUNK_SIZE]
    }
}

impl IndexMut<(usize, usize)> for Chunk {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        if index.0 >= CHUNK_SIZE { panic!("Out of bound access") }
        &mut self.blocks[index.0 + index.1 * CHUNK_SIZE]
    }
}