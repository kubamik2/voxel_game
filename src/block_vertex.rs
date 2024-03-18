use cgmath::Point3;
use wgpu::vertex_attr_array;

#[derive(Debug, Clone, Copy)]
pub struct BlockVertex {
    pub position: Point3<u8>,
    pub face: Face,
}

impl BlockVertex {
    pub fn new(position: Point3<u8>, face: Face) -> Self {
        Self { position, face }
    }

    pub fn pack(&self) -> PackedBlockVertex {
        PackedBlockVertex::new(self.position, self.face)
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    PositiveX,
    NegativeX,
    PositiveZ,
    NegativeZ,
    PositiveY,
    NegativeY,
}


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PackedBlockVertex(u32);

impl PackedBlockVertex {
    pub fn new(position: Point3<u8>, face: Face) -> Self {
        Self(position.x as u32 | (position.y as u32) << 6 | (position.z as u32) << 12 | (face as u32) << 18)
    }

    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![0 => Uint32];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PackedBlockVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBUTES,
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct VertexConstant {
    pub chunk_translation_offset: [i32; 3],
}

impl VertexConstant {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![1 => Sint32x3];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[i32; 3]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBUTES
        }
    }
}