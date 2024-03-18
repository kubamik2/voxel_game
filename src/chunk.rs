use cgmath::{Point2, Point3};
use rand::Rng;
use wgpu::util::DeviceExt;
use std::{collections::HashMap, ops::{Index, IndexMut}};

use crate::{block::*, block_vertex::VertexConstant, camera::*};

pub struct ChunkManager {
    pub chunks: HashMap<u64, Chunk>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self { chunks: HashMap::new() }
    }

    pub fn get(&self, position: Point2<i32>) -> Option<&Chunk> {
        self.chunks.get(&(position.x as u64 | (position.y as u64) << 32))
    }
}

pub struct World {
    pub loaded_chunks: ChunkManager,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub texture_atlas_bind_group: wgpu::BindGroup,
    pub depth_texture: crate::texture::Texture,
    pub material_texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl World {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, queue: &wgpu::Queue) -> Self {
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
                }
            ]
        });

        let material_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
                &material_texture_bind_group_layout,
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
                    crate::block_vertex::PackedBlockVertex::desc(),
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

        Self { camera, camera_bind_group, camera_buffer, camera_controller, camera_uniform, loaded_chunks: ChunkManager::new(), render_pipeline, texture_atlas_bind_group, depth_texture, material_texture_bind_group_layout }
    }
    pub fn generate_chunks(&mut self) {

    }

    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Surface, window: &winit::window::Window) {
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

            for chunk in self.loaded_chunks.chunks.values() {
                for sub_chunk in chunk.sub_chunks.iter() {
                    if let Some(sub_chunk) = sub_chunk {
                        render_pass.set_vertex_buffer(0, sub_chunk.mesh.vertex_buffer.slice(..));
                        render_pass.set_vertex_buffer(1, sub_chunk.translation_buffer.slice(..));

                        render_pass.set_index_buffer(sub_chunk.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                        render_pass.set_bind_group(2, &sub_chunk.material_3d_texture_bind_group, &[]);
                        
                        render_pass.draw_indexed(0..sub_chunk.mesh.indices, 0, 0..1);
                    }
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        window.pre_present_notify();
        output.present();
    }
}

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_HEIGHT: usize = 256;
pub const SUB_CHUNK_HEIGHT: usize = 32;

pub struct Chunk {
    pub position: Point2<i32>,
    pub blocks: Box<[Block]>,
    pub sub_chunks: [Option<SubChunk>; 8]
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
    pub fn load_subchunk(&mut self, index: usize, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut index_offset = 0;
        let mut vertices = vec![];
        let mut indices = vec![];
        let y_offset = index * SUB_CHUNK_HEIGHT;

        let mut i = 0;
        let mut material_data = [0; SUB_CHUNK_HEIGHT * CHUNK_SIZE * CHUNK_SIZE];

        for y in y_offset..y_offset + SUB_CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = &self[(x, y, z)];
                    material_data[i] = block.material as u8;
                    i += 1;
                    if block.material == Material::Air { continue; }
                    
                    for (i, face) in Block::FACE_VERTICES.iter().cloned().enumerate() {
                        if !self.is_face_visible(i, x, y, z) { continue; }
                        for mut vertex in face {
                            vertex.position.x += x as u8;
                            vertex.position.y += (y - y_offset) as u8;
                            vertex.position.z += z as u8;

                            vertices.push(vertex.pack());
                        }

                        for index in Block::FACE_INDICES {
                            indices.push(index + index_offset);
                        }

                        index_offset += 4;
                    }
                }
            }
        }

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

        let texture = crate::texture::Texture::create_3d_material_texture(device, queue, &material_data);
        let material_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    visibility: wgpu::ShaderStages::FRAGMENT
                },
            ]
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("3d material texture bind group"),
            layout: &material_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler)
                }
            ]
        });

        let translation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk_translation_buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&[VertexConstant { chunk_translation_offset: [self.position.x * CHUNK_SIZE as i32, y_offset as i32, self.position.y * CHUNK_SIZE as i32]}])
        });

        self.sub_chunks[index] = Some(SubChunk { mesh: ChunkMesh { vertex_buffer, index_buffer, indices: indices.len() as u32 }, material_3d_texture_bind_group: bind_group, translation_buffer })
    }

    pub fn new(position: Point2<i32>) -> Self {
        Self { 
            position,
            blocks: vec![Block { material: Material::Cobblestone }; CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT].into_boxed_slice(),
            sub_chunks: [None, None, None, None, None, None, None, None]
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

        Self { position, blocks: blocks.into_boxed_slice(), sub_chunks: [None, None, None, None, None, None, None, None] }
    }

    fn is_face_visible(&self, face_index: usize, x: usize, y: usize, z: usize) -> bool {
        
        match face_index {
            0 => {
                if x + 1 < CHUNK_SIZE {
                    self[(x + 1, y, z)].material == Material::Air
                } else {
                    true
                }
            },
            1 => {
                if x > 0 {
                    self[(x - 1, y, z)].material == Material::Air
                } else {
                    true
                }
            },
            2 => {
                if z + 1 < CHUNK_SIZE {
                    self[(x, y, z + 1)].material == Material::Air
                } else {
                    true
                }
            },
            3 => {
                if z > 0 {
                    self[(x, y, z - 1)].material == Material::Air
                } else {
                    true
                }
            },
            4 => {
                if y + 1 < CHUNK_HEIGHT {
                    self[(x, y + 1, z)].material == Material::Air
                } else {
                    true
                }
            },
            5 => {
                if y > 0 {
                    self[(x, y - 1, z)].material == Material::Air
                } else {
                    true
                }
            },
            _ => unreachable!()
        }
    }
}

pub struct SubChunk {
    pub material_3d_texture_bind_group: wgpu::BindGroup,
    pub mesh: ChunkMesh,
    pub translation_buffer: wgpu::Buffer,
}

pub struct ChunkMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices: u32,
}