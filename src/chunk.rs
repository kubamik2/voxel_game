use cgmath::{Point2, Point3};
use rand::Rng;
use wgpu::util::DeviceExt;
use std::ops::{Index, IndexMut};

use crate::{block::*, block_vertex::{BlockVertex, Face, RawBlockVertex, VertexConstant}, camera::*};

pub const RENDER_DISTANCE: usize = 1;

pub const CHUNK_SIZE: usize = 512;
pub const CHUNK_HEIGHT: usize = 256;

pub struct ChunkRenderer {
    pub chunks: Box<[Chunk]>,
}

impl ChunkRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut chunks = vec![];
        
        let mut chunk_loading_time = std::time::Duration::ZERO;
        let mut chunk_creation_time = std::time::Duration::ZERO;

        for x in 0..RENDER_DISTANCE as i32 {
            for y in 0..RENDER_DISTANCE as i32 {
                let now = std::time::Instant::now();
                let mut chunk = Chunk::perlin((x, y).into());
                chunk_creation_time += now.elapsed();

                chunk[(0, 0, 0)].material = Material::Cobblestone;


                let now = std::time::Instant::now();
                chunk.load_chunk(&device, &queue);
                chunk_loading_time += now.elapsed();

                chunks.push(chunk);
            }
        }
        println!("chunk_loading_time: {:?}", chunk_loading_time / (RENDER_DISTANCE * RENDER_DISTANCE) as u32);
        println!("chunk_creation_time: {:?}", chunk_creation_time / (RENDER_DISTANCE * RENDER_DISTANCE) as u32);

        Self { chunks: chunks.into_boxed_slice() }
    }
}

pub struct World {
    pub rendered_chunks: ChunkRenderer,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub texture_atlas_bind_group: wgpu::BindGroup,
    pub depth_texture: crate::texture::Texture,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indices: u32,
    pub gui_renderer: crate::egui_renderer::EguiRenderer,
}

impl World {
    pub fn new(window: &winit::window::Window, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, queue: &wgpu::Queue) -> Self {
        // camera
        let camera = Camera::default(config.width, config.height);

        // camera controller
        let camera_controller = CameraController::new(5.0);
        
        // camera uniform
        let camera_uniform = CameraUniform::new();

        // camera buffer
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[camera_uniform]),
        });

        // camera bind group layout
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX
                }
            ]
        });

        // camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding()
                }
            ]
        });
        
        // shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chunk shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("chunk.wgsl").into())
        });

        let texture_atlas_bytes = include_bytes!("textures/texture_atlas.png");
        let texture_atlas = crate::texture::Texture::from_bytes(&device, &queue, texture_atlas_bytes, "texture").unwrap();

        let texture_atlas_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture bind group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                }
            ]
        });

        let texture_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse bind group"),
            layout: &texture_atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_atlas.view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_atlas.sampler)
                },
            ]
        });

        let subchunk_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("3d material texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false
                    },
                    visibility: wgpu::ShaderStages::FRAGMENT
                },
            ]
        });

        // pipeline layout
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("chunk pipeline layout"),
            bind_group_layouts: &[
                &texture_atlas_bind_group_layout,
                &camera_bind_group_layout,
                &subchunk_texture_bind_group_layout,
            ],
            push_constant_ranges: &[]
        });

        // pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                buffers: &[
                    crate::block_vertex::RawBlockVertex::desc(),
                    VertexConstant::desc(),
                ],
                entry_point: "vs_main"
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None
        });

        let depth_texture = crate::texture::Texture::create_depth_texture(&device, &config, "depth texture");

        let (vertices, indices) = Self::calculate_world_mesh();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk mesh vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(vertices.as_slice())
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk mesh index buffer"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(indices.as_slice())
        });
        let indices = indices.len() as u32;

        let gui_renderer = crate::egui_renderer::EguiRenderer::new(window, config, device);

        Self { camera, camera_bind_group, camera_buffer, camera_controller, camera_uniform, rendered_chunks: ChunkRenderer::new(&device, &queue), render_pipeline, texture_atlas_bind_group, depth_texture, vertex_buffer, index_buffer, indices, gui_renderer }
    }

    pub fn calculate_world_mesh() -> (Vec<RawBlockVertex>, Vec<u32>) {
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut index_offset = 0;

        for x in 0..CHUNK_SIZE {
            for i in 0..2 {
                for mut vertex in Chunk::CHUNK_VERTICES[i] {
                    vertex.position.x += x as f32;

                    vertices.push(vertex.to_raw());
                }

                for index in Chunk::CHUNK_INDICES {
                    indices.push(index + index_offset);
                }

                index_offset += 4;
            }
        }
        for z in 0..CHUNK_SIZE {
            for i in 2..4 {
                for mut vertex in Chunk::CHUNK_VERTICES[i] {
                    vertex.position.z += z as f32;

                    vertices.push(vertex.to_raw());
                }

                for index in Chunk::CHUNK_INDICES {
                    indices.push(index + index_offset);
                }

                index_offset += 4;
            }
        }

        for y in 0..CHUNK_HEIGHT {
            for i in 4..6 {
                for mut vertex in Chunk::CHUNK_VERTICES[i] {
                    vertex.position.y += y as f32;

                    vertices.push(vertex.to_raw());
                }

                for index in Chunk::CHUNK_INDICES {
                    indices.push(index + index_offset);
                }

                index_offset += 4;
            }
        }
        
        (vertices, indices)
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue,config: &wgpu::SurfaceConfiguration, surface: &wgpu::Surface, window: &winit::window::Window, render_time: std::time::Duration, update_time: std::time::Duration) {
        let output = surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 123.0 / 255.0, g: 164.0 / 255.0, b: 1.0, a: 1.0 }),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_atlas_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            for chunk in self.rendered_chunks.chunks.iter() {
                if chunk.chunk_data_texture.is_some() && chunk.translation_buffer.is_some() {
                    render_pass.set_vertex_buffer(1, chunk.translation_buffer.as_ref().unwrap().slice(..));

                    render_pass.set_bind_group(2, chunk.chunk_data_texture.as_ref().unwrap(), &[]);
                    
                    render_pass.draw_indexed(0..self.indices, 0, 0..1);
                }
            }
        }

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point: window.scale_factor() as f32
        };
        
        self.gui_renderer.draw(device, queue, &mut encoder, window, &view, screen_descriptor, |ctx| {
            let gui = crate::gui::Gui {
                position: self.camera.eye.into(),
                direction: self.camera.direction.into(),
                render_time,
                update_time
            };

            gui.ui(ctx);
        });

        queue.submit(std::iter::once(encoder.finish()));
        window.pre_present_notify();
        output.present();
    }
}

pub struct Chunk {
    pub position: Point2<i32>,
    pub blocks: Box<[Block]>,
    pub chunk_data_texture: Option<wgpu::BindGroup>,
    pub translation_buffer: Option<wgpu::Buffer>,
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

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ChunkData(u32);

impl Chunk {
    pub const CHUNK_VERTICES: [[BlockVertex; 4]; 6] = [
        [
            BlockVertex { position: Point3::new(1.0, 0.0, (CHUNK_SIZE * RENDER_DISTANCE) as f32), face: Face::PositiveX },
            BlockVertex { position: Point3::new(1.0, 0.0, 0.0), face: Face::PositiveX },
            BlockVertex { position: Point3::new(1.0, CHUNK_HEIGHT as f32, (CHUNK_SIZE * RENDER_DISTANCE) as f32), face: Face::PositiveX },
            BlockVertex { position: Point3::new(1.0, CHUNK_HEIGHT as f32, 0.0), face: Face::PositiveX },
        ],
        [
            BlockVertex { position: Point3::new(0.0, 0.0, 0.0), face: Face::NegativeX },
            BlockVertex { position: Point3::new(0.0, 0.0, (CHUNK_SIZE * RENDER_DISTANCE) as f32), face: Face::NegativeX },
            BlockVertex { position: Point3::new(0.0, CHUNK_HEIGHT as f32, 0.0), face: Face::NegativeX },
            BlockVertex { position: Point3::new(0.0, CHUNK_HEIGHT as f32, (CHUNK_SIZE * RENDER_DISTANCE) as f32), face: Face::NegativeX },
        ],
        [
            BlockVertex { position: Point3::new(0.0, 0.0, 1.0), face: Face::PositiveZ },
            BlockVertex { position: Point3::new((CHUNK_SIZE * RENDER_DISTANCE) as f32, 0.0, 1.0), face: Face::PositiveZ },
            BlockVertex { position: Point3::new(0.0, CHUNK_HEIGHT as f32, 1.0), face: Face::PositiveZ },
            BlockVertex { position: Point3::new((CHUNK_SIZE * RENDER_DISTANCE) as f32, CHUNK_HEIGHT as f32, 1.0), face: Face::PositiveZ },
        ],
        [
            BlockVertex { position: Point3::new((CHUNK_SIZE * RENDER_DISTANCE) as f32, 0.0, 0.0), face: Face::NegativeZ },
            BlockVertex { position: Point3::new(0.0, 0.0, 0.0), face: Face::NegativeZ },
            BlockVertex { position: Point3::new((CHUNK_SIZE * RENDER_DISTANCE) as f32, CHUNK_HEIGHT as f32, 0.0), face: Face::NegativeZ },
            BlockVertex { position: Point3::new(0.0, CHUNK_HEIGHT as f32, 0.0), face: Face::NegativeZ },
        ],
        [
            BlockVertex { position: Point3::new(0.0, 1.0, 0.0), face: Face::PositiveY },
            BlockVertex { position: Point3::new(0.0, 1.0, (CHUNK_SIZE * RENDER_DISTANCE) as f32), face: Face::PositiveY },
            BlockVertex { position: Point3::new((CHUNK_SIZE * RENDER_DISTANCE) as f32, 1.0, 0.0), face: Face::PositiveY },
            BlockVertex { position: Point3::new((CHUNK_SIZE * RENDER_DISTANCE) as f32, 1.0, (CHUNK_SIZE * RENDER_DISTANCE) as f32), face: Face::PositiveY },
        ],
        [
            BlockVertex { position: Point3::new(CHUNK_SIZE as f32, 0.0, 0.0), face: Face::NegativeY },
            BlockVertex { position: Point3::new(CHUNK_SIZE as f32, 0.0, CHUNK_SIZE as f32), face: Face::NegativeY },
            BlockVertex { position: Point3::new(0.0, 0.0, 0.0), face: Face::NegativeY },
            BlockVertex { position: Point3::new(0.0, 0.0, CHUNK_SIZE as f32), face: Face::NegativeY },
        ],
    ];

    pub const CHUNK_INDICES: [u32; 6] = [
        2, 0, 1,
        3, 2, 1
    ];

    #[inline]
    pub fn load_chunk(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut chunk_data = Box::new([ChunkData(0); CHUNK_HEIGHT * CHUNK_SIZE * CHUNK_SIZE]);

        let mut i = 0;
        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let material = self[(x, y, z)].material;
                    let face_visibility_bitmask = 
                    (self.is_face_visible(0, x, y, z) as u8) | 
                    (self.is_face_visible(1, x, y, z) as u8) << 1 | 
                    (self.is_face_visible(2, x, y, z) as u8) << 2 | 
                    (self.is_face_visible(3, x, y, z) as u8) << 3 | 
                    (self.is_face_visible(4, x, y, z) as u8) << 4 | 
                    (self.is_face_visible(5, x, y, z) as u8) << 5;

                    chunk_data[i] = ChunkData(material as u32 | (face_visibility_bitmask as u32) << 8);

                    i += 1;
                }
            }
        }

        let chunk_data_texture = crate::texture::Texture::create_3d_material_texture(device, queue, bytemuck::cast_slice(&*chunk_data));
        let chunk_data_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("subchunk texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false
                    },
                    visibility: wgpu::ShaderStages::FRAGMENT
                },
            ]
        });
        let chunk_data_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("3d material texture bind group"),
            layout: &chunk_data_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&chunk_data_texture.view)
                },
            ]
        });

        let translation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk_translation_buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&[VertexConstant { chunk_translation_offset: [self.position.x * CHUNK_SIZE as i32, 0, self.position.y * CHUNK_SIZE as i32]}])
        });

        self.chunk_data_texture = Some(chunk_data_texture_bind_group);
        self.translation_buffer = Some(translation_buffer);
    }

    pub fn new(position: Point2<i32>) -> Self {
        Self { 
            position,
            blocks: vec![Block { material: Material::Cobblestone }; CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT].into_boxed_slice(),
            chunk_data_texture: None,
            translation_buffer: None
        }
    }

    pub fn randomized(position: Point2<i32>) -> Self {
        let mut blocks = vec![];

        let mut rng = rand::thread_rng();

        for _ in 0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT {
            let material = match rng.gen_range(0..4) {
                0 => Material::Air,
                1 => Material::Cobblestone,
                2 => Material::Dirt,
                3 => Material::Grass,
                _ => unreachable!()
            };
    
            blocks.push(Block { material });
        }

        for y in 64..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    blocks[x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE].material = Material::Air;
                }
            }
        }


        Self { position, blocks: blocks.into_boxed_slice(), chunk_data_texture: None, translation_buffer: None }
    }

    
    pub fn perlin(position: Point2<i32>) -> Self {
        let mut blocks = vec![];
        use noise::{NoiseFn, Perlin};
        let perlin1 = Perlin::new(643636362);
        let perlin2 = Perlin::new(260834963);

        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let mut val = perlin1.get([((position.x * CHUNK_SIZE as i32) as f64 + x as f64) / (64) as f64, ((position.y * CHUNK_SIZE as i32) as f64 + z as f64) / (64) as f64]);
                    val += 8.0 * (perlin2.get([((position.x * CHUNK_SIZE as i32) as f64 + x as f64) / (64 * 10) as f64, ((position.y * CHUNK_SIZE as i32) as f64 + z as f64) / (64 * 10) as f64])).max(0.0);
                    
                    if (val * CHUNK_HEIGHT as f64 * 0.1 + 64.0) > y as f64 {
                        blocks.push(Block { material: Material::Grass });
                    } else {
                        blocks.push(Block { material: Material::Air });
                    }
                }
            }
        }

        Self { position, blocks: blocks.into_boxed_slice(), chunk_data_texture: None, translation_buffer: None }
    }

    #[inline]
    fn is_face_visible(&self, face_index: usize, x: usize, y: usize, z: usize) -> bool {
        
        match face_index {
            0 => {
                !(x + 1 < CHUNK_SIZE) || self[(x + 1, y, z)].material == Material::Air
            },
            1 => {
                !(x > 0) || self[(x - 1, y, z)].material == Material::Air
            },
            2 => {
                !(z + 1 < CHUNK_SIZE) || self[(x, y, z + 1)].material == Material::Air
            },
            3 => {
                !(z > 0) || self[(x, y, z - 1)].material == Material::Air
            },
            4 => {
                !(y + 1 < CHUNK_HEIGHT) || self[(x, y + 1, z)].material == Material::Air
            },
            5 => {
                !(y > 0) || self[(x, y - 1, z)].material == Material::Air
            },
            _ => unreachable!()
        }
    }
}