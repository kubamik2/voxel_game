use cgmath::Point3;
use wgpu::vertex_attr_array;

#[derive(Debug, Clone, Copy)]
pub struct BlockVertex {
    pub position: Point3<u8>, // u1
    pub chunk_index: u8,
}

impl BlockVertex {
    pub fn new(position: Point3<u8>, face: Face, texture_index: u16) -> Self {
        Self { position, chunk_index: 0 }
    }

    pub fn to_raw(&self) -> RawBlockVertex {
        let mut packed_data = 0;

        packed_data = self.position.x as u32;
        packed_data |= (self.position.y as u32) << 1;
        packed_data |= (self.position.z as u32) << 2;
        packed_data |= (self.chunk_index as u32) << 3;

        RawBlockVertex(packed_data)
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
pub struct RawBlockVertex(pub u32);

impl RawBlockVertex {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &vertex_attr_array![0 => Uint32];
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
pub struct ChunkTranslation {
    pub chunk_translation_offset: [f32; 2],
}

impl ChunkTranslation {
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("translation_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    visibility: wgpu::ShaderStages::VERTEX
                }
            ]
        })
    }
}