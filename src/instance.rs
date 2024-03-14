#[derive(Debug, Clone, Copy)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub texture_offset: cgmath::Vector2<f32>
}

impl From<Instance> for InstanceRaw {
    fn from(value: Instance) -> Self {
        Self { model: (cgmath::Matrix4::from_translation(value.position) * cgmath::Matrix4::from(value.rotation)).into(), texture_offset: value.texture_offset.into() }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    texture_offset: [f32; 2]
}

impl InstanceRaw {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem::size_of;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBUTES
        }
    }
}