use cgmath::{Point2, Point3};
use wgpu::vertex_attr_array;

use crate::block_vertex::Face;

pub struct BlockFaceInstance {
    pub position: Point3<u8>, // u12
    pub face: Face, // u3
    pub texture_index: u16, // u?
    pub greedy_tiling: Point2<u8> // u8
}

impl BlockFaceInstance {
    pub fn to_raw(&self) -> BlockFaceInstanceRaw {
        let mut packed_data = 0;
        
        packed_data = self.position.x as u32;
        packed_data |= (self.position.y as u32) << 4;
        packed_data |= (self.position.z as u32) << 8;
        packed_data |= (self.face as u32) << 12;
        packed_data |= (self.texture_index as u32) << 15;
        packed_data |= (self.greedy_tiling.x as u32) << 23;
        packed_data |= (self.greedy_tiling.y as u32) << 27;

        BlockFaceInstanceRaw(packed_data)
    }
}


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct BlockFaceInstanceRaw(pub u32);

impl BlockFaceInstanceRaw {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![1 => Uint32];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBUTES
        }
    }
}

