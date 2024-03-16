pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;
use std::ops::{Index, IndexMut};

use crate::{block::{Block, Material}, vertex::{Vertex, VertexPacked}};

pub struct Chunk {
    pub blocks: Vec<Block>
}

impl Index<(usize, usize, usize)> for Chunk {
    type Output = Block;
    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        &self.blocks[index.0 + index.1 * CHUNK_SIZE * CHUNK_SIZE + index.2 * CHUNK_SIZE]
    }
}

impl IndexMut<(usize, usize, usize)> for Chunk {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        &mut self.blocks[index.0 + index.1 * CHUNK_SIZE * CHUNK_SIZE + index.2 * CHUNK_SIZE]
    }
}

impl Chunk {
    pub fn push_vertex_data(&self, vertex_array: &mut Vec<VertexPacked>, index_array: &mut Vec<u32>, mut index_offset: u32) -> u32 {
        for block in self.blocks.iter().filter(|p| p.material != Material::Air) {
            let texture_offsets = block.material.texture_offsets();
            for (i, mut face) in Block::FACE_VERTICES.iter().cloned().enumerate() {
                if !block.bitmap.get(i) {
                    for vertex in face.iter_mut() {
                        vertex.position[0] += block.position.x;
                        vertex.position[1] += block.position.y;
                        vertex.position[2] += block.position.z;

                        vertex.tex_coords[0] += texture_offsets[i].x;
                        vertex.tex_coords[1] += texture_offsets[i].y;

                        vertex_array.push(vertex.pack());
                    }
    
                    for index in Block::FACE_INDICES {
                        index_array.push(index + index_offset);
                    }

                    index_offset += 4;
                }
            }
        }

        index_offset
    }

    
    pub fn bake(&mut self) {
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
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

                    if y + 1 < CHUNK_HEIGHT {
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

pub const WORLD_SIZE: usize = 24;

pub struct World {
    pub chunks: Vec<Chunk>
}

impl Index<(usize, usize)> for World {
    type Output = Chunk;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.chunks[index.0 + index.1 * WORLD_SIZE]
    }
}

impl IndexMut<(usize, usize)> for World {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.chunks[index.0 + index.1 * WORLD_SIZE]
    }
}

impl World {
    pub fn new() -> Self {
        let mut chunks = vec![];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for iz in 0..WORLD_SIZE {
            let iz = iz as f32;
            for ix in 0..WORLD_SIZE {
                let ix = ix as f32;

                let mut blocks = vec![];
                for y in 0..CHUNK_HEIGHT {
                    for z in 0..CHUNK_SIZE {
                        for x in 0..CHUNK_SIZE {
                            let material = if rng.gen() { Material::Grass } else { Material::Air };
                            blocks.push(Block { position: cgmath::Point3::new(x as f32 + ix * CHUNK_SIZE as f32, y as f32, z as f32 + iz * CHUNK_SIZE as f32), bitmap: crate::block::Bitmap(0), material });
                        }
                    }
                }
    
                chunks.push(Chunk { blocks });
            }
        }

        Self { chunks }
    }

    pub fn bake_chunk(&mut self, position: (usize, usize)) {
        self[position].bake();

        if position.1 > 0 {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_HEIGHT {
                    let v = self[(position.0, position.1 - 1)][(x, y, CHUNK_SIZE - 1)].material != Material::Air;
                    self[position][(x, y, 0)].bitmap.set(3, v);
                }
            }
        }

        if position.1 + 1 < WORLD_SIZE {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_HEIGHT {
                    let v = self[(position.0, position.1 + 1)][(x, y, 0)].material != Material::Air;
                    self[position][(x, y, CHUNK_SIZE - 1)].bitmap.set(2, v);
                }
            }
        }


        if position.0 > 0 {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_HEIGHT {
                    let v = self[(position.0 - 1, position.1)][(CHUNK_SIZE - 1, y, z)].material != Material::Air;
                    self[position][(0, y, z)].bitmap.set(1, v);
                }
            }
        }

        if position.0 + 1 < WORLD_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_HEIGHT {
                    let v = self[(position.0 + 1, position.1)][(0, y, z)].material != Material::Air;
                    self[position][(CHUNK_SIZE - 1, y, z)].bitmap.set(0, v);
                }
            }
        }
    }
}