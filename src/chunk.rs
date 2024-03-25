use cgmath::{num_traits::Pow, Point2, Point3};
use rand::Rng;
use wgpu::util::{DeviceExt, RenderEncoder};
use std::ops::{Index, IndexMut};

use crate::{block::*, block_vertex::{BlockVertex, ChunkTranslation, Face, RawBlockVertex}, camera::*, instance::{BlockFaceInstance, BlockFaceInstanceRaw}};

pub const RENDER_DISTANCE: usize = 32;

pub const CHUNK_SIZE: usize = 16;
pub const WORLD_HEIGHT: usize = 384;

pub const CHUNKS_PER_WORLD_CHUNK: usize = WORLD_HEIGHT / CHUNK_SIZE;

pub struct ChunkManager {
    pub chunks: Box<[WorldChunk]>,
}

impl ChunkManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut chunks = vec![];

        let mut chunk_construction_time = std::time::Duration::ZERO;
        for z in 0..RENDER_DISTANCE {
            for x in 0..RENDER_DISTANCE {
                let chunk = WorldChunk::perlin([x as i32, z as i32].into(), device);
                
                chunks.push(chunk);
            }
        }

        let mut manager = Self { chunks: chunks.into_boxed_slice() };
        let now = std::time::Instant::now();
        manager.bake_chunks();
        let chunk_baking_time = now.elapsed();

        for chunk in manager.chunks.iter_mut() {
            let now = std::time::Instant::now();
            for i in 0..CHUNKS_PER_WORLD_CHUNK {
                chunk.mesh_chunk(i, queue);
            }
            chunk_construction_time += now.elapsed();
        }

        println!("chunk_construction_time: {:?}", chunk_construction_time / (RENDER_DISTANCE.pow(2) * CHUNKS_PER_WORLD_CHUNK) as u32);
        println!("world_chunk_construction_time: {:?}", chunk_construction_time / (RENDER_DISTANCE.pow(2)) as u32);
        println!("chunk_baking_time: {:?}", chunk_baking_time);
        manager
    }
    
    pub fn bake_chunks(&mut self) {
        for chunk_y in 0..RENDER_DISTANCE {
            for chunk_x in 0..RENDER_DISTANCE {
                let world_chunk = &mut self.chunks[chunk_y * RENDER_DISTANCE + chunk_x];

                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        for chunk_index in 0..CHUNKS_PER_WORLD_CHUNK {
                            if chunk_index + 1 < CHUNKS_PER_WORLD_CHUNK {
                                let v = world_chunk.chunks[chunk_index + 1][(x, 0, z)].material == Material::Air;
                                world_chunk.chunks[chunk_index][(x, CHUNK_SIZE - 1, z)].adjacent_blocks_bitmap.set_face(4, v);
                            }

                            if chunk_index > 0 {
                                let v = world_chunk.chunks[chunk_index - 1][(x, CHUNK_SIZE - 1, z)].material == Material::Air;
                                world_chunk.chunks[chunk_index][(x, 0, z)].adjacent_blocks_bitmap.set_face(5, v);
                            }
                        }
                    }
                }

                if chunk_x + 1 < RENDER_DISTANCE {
                    for chunk_index in 0..CHUNKS_PER_WORLD_CHUNK {
                        for y in 0..CHUNK_SIZE {
                            for z in 0..CHUNK_SIZE {
                                let v = self.chunks[chunk_y * RENDER_DISTANCE + chunk_x + 1].chunks[chunk_index][(0, y, z)].material == Material::Air;
                                self.chunks[chunk_y * RENDER_DISTANCE + chunk_x].chunks[chunk_index][(CHUNK_SIZE - 1, y, z)].adjacent_blocks_bitmap.set_face(0, v);
                            }
                        }
                    }
                }

                if chunk_x > 0 {
                    for chunk_index in 0..CHUNKS_PER_WORLD_CHUNK {
                        for y in 0..CHUNK_SIZE {
                            for z in 0..CHUNK_SIZE {
                                let v = self.chunks[chunk_y * RENDER_DISTANCE + chunk_x - 1].chunks[chunk_index][(CHUNK_SIZE - 1, y, z)].material == Material::Air;
                                self.chunks[chunk_y * RENDER_DISTANCE + chunk_x].chunks[chunk_index][(0, y, z)].adjacent_blocks_bitmap.set_face(1, v);
                            }
                        }
                    }
                }

                if chunk_y + 1 < RENDER_DISTANCE {
                    for chunk_index in 0..CHUNKS_PER_WORLD_CHUNK {
                        for y in 0..CHUNK_SIZE {
                            for x in 0..CHUNK_SIZE {
                                let v = self.chunks[(chunk_y + 1) * RENDER_DISTANCE + chunk_x].chunks[chunk_index][(x, y, 0)].material == Material::Air;
                                self.chunks[chunk_y * RENDER_DISTANCE + chunk_x].chunks[chunk_index][(x, y, CHUNK_SIZE - 1)].adjacent_blocks_bitmap.set_face(2, v);
                            }
                        }
                    }
                }

                if chunk_y > 0 {
                    for chunk_index in 0..CHUNKS_PER_WORLD_CHUNK {
                        for y in 0..CHUNK_SIZE {
                            for x in 0..CHUNK_SIZE {
                                let v = self.chunks[(chunk_y - 1) * RENDER_DISTANCE + chunk_x].chunks[chunk_index][(x, y, CHUNK_SIZE - 1)].material == Material::Air;
                                self.chunks[chunk_y * RENDER_DISTANCE + chunk_x].chunks[chunk_index][(x, y, 0)].adjacent_blocks_bitmap.set_face(3, v);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct World {
    pub rendered_chunks: ChunkManager,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub texture_atlas_bind_group: wgpu::BindGroup,
    pub depth_texture: crate::texture::Texture,
    pub gui_renderer: crate::egui_renderer::EguiRenderer,
    pub vertex_buffer: wgpu::Buffer,
    pub indirect_buffer: wgpu::Buffer,
    pub world_buffer: wgpu::Buffer,
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

        let translation_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        });

        // pipeline layout
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("chunk pipeline layout"),
            bind_group_layouts: &[
                &texture_atlas_bind_group_layout,
                &camera_bind_group_layout,
                &translation_bind_group_layout
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
                    crate::instance::BlockFaceInstanceRaw::desc(),
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
                topology: wgpu::PrimitiveTopology::TriangleStrip,
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

        let gui_renderer = crate::egui_renderer::EguiRenderer::new(window, config, device);

        let mut vertices = [RawBlockVertex(0); CHUNKS_PER_WORLD_CHUNK * Block::FACE_VERTICES.len()];
        for i in 0..CHUNKS_PER_WORLD_CHUNK {
            for (j, mut vertex) in Block::FACE_VERTICES.iter().cloned().enumerate() {
                vertex.chunk_index = i as u8;
                vertices[i * Block::FACE_VERTICES.len() + j] = vertex.to_raw();
            }
        };
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&vertices)
        });

        let indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indirect_buffer"),
            usage: wgpu::BufferUsages::INDIRECT,
            contents: wgpu::util::DrawIndirect  {
                base_instance: 0,
                base_vertex: 0,
                vertex_count: Block::FACE_VERTICES.len() as u32,
                instance_count: 10000,
            }.as_bytes()
        });
        // println!("{}", ((CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) / 2) * RENDER_DISTANCE * RENDER_DISTANCE * 6 * 4 * 8);
        let world_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("world_buffer"),
            mapped_at_creation: false,
            size: 0,//(((CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) / 2) * RENDER_DISTANCE * RENDER_DISTANCE * 6 * 4 * 8) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self { camera, camera_bind_group, camera_buffer, camera_controller, camera_uniform, rendered_chunks: ChunkManager::new(&device, &queue), render_pipeline, texture_atlas_bind_group, depth_texture, gui_renderer, vertex_buffer, indirect_buffer, world_buffer }
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
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.texture_atlas_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            

            for chunk in self.rendered_chunks.chunks.iter() {
                render_pass.set_vertex_buffer(1, chunk.mesh.instance_buffer.slice(..));
                render_pass.set_bind_group(2, &chunk.mesh.translation_bind_group, &[]);

                render_pass.multi_draw_indirect(&chunk.mesh.indirect_buffer, 0, CHUNKS_PER_WORLD_CHUNK as u32);
                
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

pub struct WorldChunk {
    pub position: Point2<i32>,
    pub chunks: Box<[Chunk]>,
    pub mesh: WorldChunkMesh,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub blocks: Box<[Block]>,
}

pub const MAX_CHUNK_BUCKET_SIZE: u32 = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 3 * std::mem::size_of::<BlockFaceInstanceRaw>()) as u32;
pub const MIN_CHUNK_BUCKET_SIZE: u32 = MAX_CHUNK_BUCKET_SIZE / 32;

pub struct WorldChunkMesh {
    pub instance_buffer: wgpu::Buffer,
    pub indirect_buffer: wgpu::Buffer,
    pub chunk_bucket_sizes: [u32; CHUNKS_PER_WORLD_CHUNK],
    pub chunk_bucket_instances_count: [u32; CHUNKS_PER_WORLD_CHUNK],
    pub translation_bind_group: wgpu::BindGroup,
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

impl WorldChunk {
    pub fn randomized(position: Point2<i32>, device: &wgpu::Device) -> Self {
        let chunks: Box<[Chunk; CHUNKS_PER_WORLD_CHUNK]> = Box::new(std::array::from_fn(|_| Chunk::randomized()));
        Self {
            chunks,
            mesh: Self::initialize_mesh(device, position),
            position
        }
    }

    pub fn perlin(position: Point2<i32>, device: &wgpu::Device) -> Self {
        use noise::NoiseFn;
        let perlin = noise::Perlin::new(626645783);
        let mut chunks = vec![];
        for i in 0..CHUNKS_PER_WORLD_CHUNK {
            let mut blocks = vec![];
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let val = (perlin.get([(x as i32 + position.x * CHUNK_SIZE as i32) as f64 / 64.0, (z as i32 + position.y * CHUNK_SIZE as i32) as f64 / 64.0]).clamp(0.0, WORLD_HEIGHT as f64) * 50.0) as usize + 100;
                        if y + i * CHUNK_SIZE < val {
                            blocks.push(Block { material: Material::Grass, adjacent_blocks_bitmap: AdjacentBlockBitmap(u8::MAX) });
                        } else {
                            blocks.push(Block { material: Material::Air, adjacent_blocks_bitmap: AdjacentBlockBitmap(u8::MIN) });
                        }
                    }
                }
            }
            let mut chunk = Chunk { blocks: blocks.into_boxed_slice() };
            chunk.bake_faces();
            chunks.push(chunk);
        }

        Self { position, chunks: chunks.into_boxed_slice(), mesh: Self::initialize_mesh(device, position) }
    }

    pub fn initialize_mesh(device: &wgpu::Device, position: Point2<i32>) -> WorldChunkMesh {
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: 1_179_648, // TODO change
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
        });

        let indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: 384, // TODO change
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST
        });

        let translation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("translation_buffer"),
            usage: wgpu::BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[
                ChunkTranslation {
                    chunk_translation_offset: [
                        (position.x * CHUNK_SIZE as i32) as f32,
                        (position.y * CHUNK_SIZE as i32) as f32,
                        ] 
                    }
                ])
        });

        let translation_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        });

        let translation_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("translation_bind_group"),
            layout: &translation_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: translation_buffer.as_entire_binding()
                }
            ]
        });

        WorldChunkMesh {
            instance_buffer,
            indirect_buffer,
            chunk_bucket_sizes: std::array::from_fn(|_| (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 3 * 4) as u32),
            chunk_bucket_instances_count: std::array::from_fn(|_| 0),
            translation_bind_group
        }
    }

    pub fn mesh_chunk(&mut self, index: usize, queue: &wgpu::Queue) {
        let mut instances = [BlockFaceInstanceRaw(0); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 3];
        let mut i = 0;
        let chunk = &self.chunks[index];
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = &chunk[(x, y, z)];
                    if block.material == Material::Air { continue; }
                    for face in 0..6 {
                        if !block.adjacent_blocks_bitmap.face_visible(face) { continue; }
                        instances[i] = BlockFaceInstance { 
                            position: Point3::new(x as u8, y as u8, z as u8),
                            face: unsafe { std::mem::transmute::<u32, Face>(face as u32) },
                            material_index: block.material as u16,
                        }.to_raw();
                        i += 1;
                    }
                }
            }
        }
        
        self.mesh.chunk_bucket_instances_count[index] = i as u32;

        let instance_data: &[u8] = bytemuck::cast_slice(&instances[0..i]);


        let offset: u32 = (0..index).map(|f| self.mesh.chunk_bucket_sizes[f]).sum();
        let size = self.mesh.chunk_bucket_sizes[index];

        queue.write_buffer(&self.mesh.instance_buffer, offset as u64, instance_data);

        let base_instance = (0..index).map(|f| self.mesh.chunk_bucket_sizes[f]).sum::<u32>() / std::mem::size_of::<BlockFaceInstanceRaw>() as u32;
        let indirect_args = wgpu::util::DrawIndirect {
            base_instance,
            base_vertex: (index * Block::FACE_VERTICES.len()) as u32,
            instance_count: i as u32,
            vertex_count: Block::FACE_VERTICES.len() as u32,
        };

        queue.write_buffer(&self.mesh.indirect_buffer, (index * std::mem::size_of::<wgpu::util::DrawIndirect>()) as u64, indirect_args.as_bytes())
    }

    pub fn remesh_world_chunk(&mut self) {

    }
}

impl Chunk {
    pub fn chunk_translation_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("chunk_translation_bind_group_layout"),
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

    pub fn new() -> Self {
        let mut chunk = Self { 
            blocks: vec![Block { material: Material::Cobblestone, adjacent_blocks_bitmap: AdjacentBlockBitmap(u8::MAX) }; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE].into_boxed_slice(),
        };

        chunk.bake_faces();
        chunk
    }

    pub fn randomized() -> Self {
        let mut blocks = vec![];
        let mut rng = rand::thread_rng();

        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let material = match rng.gen_range(0..2) {
                        0 => Material::Air,
                        1 => Material::Cobblestone,
                        _ => unreachable!()
                    };

                    blocks.push(Block { material, adjacent_blocks_bitmap: AdjacentBlockBitmap(u8::MAX) })
                }
            }
        }
        let mut chunk = Chunk { blocks: blocks.into_boxed_slice() };
        chunk.bake_faces();
        chunk
    }

    pub fn empty() -> Self {
        let mut blocks = vec![];

        for i in 0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE {
            blocks.push(Block { material: Material::Air, adjacent_blocks_bitmap: AdjacentBlockBitmap(u8::MAX)})
        }

        Self { blocks: blocks.into_boxed_slice() }
    }

    pub fn bake_faces(&mut self) {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    for face in 0..6 {
                        let v = self.is_face_visible(face, x, y, z);
                        self[(x, y, z)].adjacent_blocks_bitmap.set_face(face, v);
                    }
                }
            }
        }
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
                !(y + 1 < CHUNK_SIZE) || self[(x, y + 1, z)].material == Material::Air
            },
            5 => {
                !(y > 0) || self[(x, y - 1, z)].material == Material::Air
            },
            _ => unreachable!()
        }
    }
}