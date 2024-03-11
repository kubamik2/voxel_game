pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    len: usize,
    texture: crate::texture::Texture,
    texture_bind_group: wgpu::BindGroup,
}