pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler
}

impl Texture {
    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8], label: &str) -> anyhow::Result<Self> {
        let image = image::load_from_memory(bytes)?;
        Ok(Self::from_image(device, queue, image, Some(label)))
    }

    pub fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, image: image::DynamicImage, label: Option<&str>) -> Self {
        let rgba = image.to_rgba8();

        use image::GenericImageView;
        let dimensions = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            sample_count: 1
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self { texture, view, sampler }
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            sample_count: 1,
            view_formats: &[]
        };

        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(label),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self { texture, view, sampler }
    }

    pub fn create_3d_material_texture(device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) -> Self {
        let size = wgpu::Extent3d {
            width: crate::chunk::CHUNK_SIZE as u32,
            height: crate::chunk::CHUNK_SIZE as u32,
            depth_or_array_layers: crate::chunk::SUB_CHUNK_HEIGHT as u32,
        };

        let texture = device.create_texture(&Self::desc_3d_material_texture());

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(crate::chunk::CHUNK_SIZE as u32),
                rows_per_image: Some(crate::chunk::CHUNK_SIZE as u32),
            },
            size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("3d material texture"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self { texture, view, sampler }
    }

    pub fn desc_3d_material_texture() -> wgpu::TextureDescriptor<'static> {
        let size = wgpu::Extent3d {
            width: crate::chunk::CHUNK_SIZE as u32,
            height: crate::chunk::CHUNK_SIZE as u32,
            depth_or_array_layers: crate::chunk::SUB_CHUNK_HEIGHT as u32,
        };
        wgpu::TextureDescriptor {
            label: Some("3d material texture"),
            size,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R8Uint,
            sample_count: 1,
            view_formats: &[]
        }
    }
}