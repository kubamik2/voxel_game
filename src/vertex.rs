use cgmath::Point3;
use wgpu::vertex_attr_array;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [u8; 2],
    pub tile: [u8; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    shader_location: 1,
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }

    pub fn pack(&self) -> VertexPacked {
        VertexPacked::new(self.position, self.tex_coords, self.tile)
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PackedVertexData(u32);

impl PackedVertexData {
    pub fn new(tex_coords: [u8; 2], tile: [u8; 2]) -> Self {
        Self((tex_coords[0] as u32) << 24 | (tex_coords[1] as u32) << 16 | (tile[0] as u32) << 8 | tile[1] as u32)
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct VertexPacked {
    pub position: [f32; 3],
    pub packed_vertex_data: PackedVertexData
}

impl VertexPacked {
    pub fn new(position: [f32; 3], tex_coords: [u8; 2], tile: [u8; 2]) -> Self {
        Self { position, packed_vertex_data: PackedVertexData::new(tex_coords, tile)}
    }

    pub const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![0 => Float32x3, 1 => Uint32];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VertexPacked>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBUTES
        }
    }
}