use cgmath::Point3;
use wgpu::vertex_attr_array;

#[derive(Debug, Clone, Copy)]
pub struct BlockVertex {
    pub position: Point3<f32>,
    pub face: Face,
}

impl BlockVertex {
    pub fn new(position: Point3<f32>, face: Face) -> Self {
        Self { position, face }
    }

    pub fn to_raw(&self) -> RawBlockVertex {
        RawBlockVertex { position: self.position.into(), face: self.face as u32 }
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
pub struct RawBlockVertex {
    position: [f32; 3],
    face: u32
}

impl RawBlockVertex {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![0 => Float32x3, 1 => Uint32];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RawBlockVertex>() as wgpu::BufferAddress,
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
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![7 => Sint32x3];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[i32; 3]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBUTES
        }
    }
}