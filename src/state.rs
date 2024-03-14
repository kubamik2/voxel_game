use cgmath::{Point3, Rotation3};
use winit::window::Window;
use crate::{block::{Bitmap, Block}, chunk::Chunk, vertex::Vertex};
use wgpu::util::DeviceExt;

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub diffuse_bind_group: wgpu::BindGroup,
    pub camera: crate::camera::Camera,
    pub camera_controller: crate::camera::CameraController,
    pub camera_uniform: crate::camera::CameraUniform,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_buffer: wgpu::Buffer,
    pub instances: Vec<crate::instance::Instance>,
    pub instance_buffer: wgpu::Buffer,
    pub depth_texture: crate::texture::Texture,
}

// const VERTICES: &'static [Vertex] = &[
//     Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 1.0] }, // ▄ 
//     Vertex { position: [1.0, 0.0, 0.0], tex_coords: [1.0, 1.0] }, //  ▄
//     Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0] }, // ▀
//     Vertex { position: [1.0, 1.0, 0.0], tex_coords: [1.0, 0.0] }, //  ▀

//     Vertex { position: [1.0, 0.0, -1.0], tex_coords: [0.0, 1.0] },
//     Vertex { position: [1.0, 1.0, -1.0], tex_coords: [0.0, 0.0] },
// ];

// const INDICES: &'static [u16] = &[
//     0, 1, 2,
//     1, 3, 2,

//     3, 1, 4,
//     3, 4, 5
// ];

const VERTICES: &'static [Vertex] = &[
    Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0625] }, // ▄ 
    Vertex { position: [1.0, 0.0, 0.0], tex_coords: [0.0625, 0.0625] }, //  ▄
    Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0.0, 0.0] }, // ▀
    Vertex { position: [1.0, 1.0, 0.0], tex_coords: [0.0625, 0.0] }, //  ▀

    Vertex { position: [1.0, 0.0, -1.0], tex_coords: [0.0, 0.0625] },
    Vertex { position: [1.0, 1.0, -1.0], tex_coords: [0.0, 0.0] },
];

const INDICES: &'static [u16] = &[
    0, 1, 2,
    1, 3, 2,

    3, 1, 4,
    3, 4, 5
];

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default() | wgpu::Features::POLYGON_MODE_LINE,
            limits: wgpu::Limits::default()
        }, None).await.unwrap();
        dbg!(wgpu::Limits::default().max_buffer_size);
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|p| p.is_srgb()).unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };

        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("textures/texture_atlas.png");
        let texture = crate::texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "texture").unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse bind group"),
            layout: &texture_bind_group_layout,
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

        let depth_texture = crate::texture::Texture::create_depth_texture(&device, &config, "depth texture");

        let mut camera = crate::camera::Camera::default(config.width, config.height);

        let mut camera_uniform = crate::camera::CameraUniform::new();
        camera_uniform.update_view_projection(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[camera_uniform])
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ]
        });

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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("test shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
        });

        

        let mut instances = Vec::with_capacity(100_000);


        let now = std::time::Instant::now();

        let mut chunks = vec![];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for ix in 0..8 {
            for iz in 0..8 {
                let mut blocks = Vec::with_capacity(16 * 16 * 256);
                for y in 0..128 {
                    for z in 0..16 {
                        for x in 0..16 {
                            let material = if rng.gen::<bool>() { crate::block::Material::Grass } else { crate::block::Material::Air };
                            blocks.push(Block { position: Point3::new(x as f32 + ix as f32 * 16.0, y as f32, z as f32 + iz as f32 * 16.0), bitmap: Bitmap(0b00000000), material });
                        }
                    }
                }

                for y in 128..256 {
                    for z in 0..16 {
                        for x in 0..16 {
                            blocks.push(Block { position: Point3::new(x as f32 + ix as f32 * 16.0, y as f32, z as f32 + iz as f32 * 16.0), bitmap: Bitmap(0b00000000), material: crate::block::Material::Air });
                        }
                    }
                }
                chunks.push(Chunk { blocks });
            }
        }
        println!("chunk_gen: {:?}", now.elapsed());
        
        

        let now = std::time::Instant::now();
        for chunk in chunks.iter_mut() {
            chunk.bake();
        }  
        println!("chunk_baking: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        for chunk in chunks.iter() {
            for block in chunk.blocks.iter().filter(|p| p.material != crate::block::Material::Air) {
                let Point3 { x, y, z } = block.position;
                let position1 = cgmath::Vector3::new(x as f32, y as f32, z as f32);
                let position2 = cgmath::Vector3::new(x as f32 + 1.0, y as f32, z as f32);
                let position3 = cgmath::Vector3::new(x as f32 + 1.0, y as f32, z as f32 - 1.0);
                let position4 = cgmath::Vector3::new(x as f32, y as f32, z as f32 - 1.0);
                let position5 = cgmath::Vector3::new(x as f32, y as f32 + 1.0, z as f32);
                let position6 = cgmath::Vector3::new(x as f32, y as f32, z as f32 - 1.0);
                
                let texture_offsets = block.material.texture_offsets();
    
                if !block.positive_z_occupied() {
                    instances.push(crate::instance::Instance { position: position1, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(0.0)), texture_offset: texture_offsets[0] });
                }
    
                if !block.positive_x_occupied() {
                    instances.push(crate::instance::Instance { position: position2, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0)), texture_offset: texture_offsets[1] });
                }
    
                if !block.negative_z_occupied() {
                    instances.push(crate::instance::Instance { position: position3, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(180.0)), texture_offset: texture_offsets[2] });
                }
                
                if !block.negative_x_occupied() {
                    instances.push(crate::instance::Instance { position: position4, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(-90.0)), texture_offset: texture_offsets[3] });
                }
    
                if !block.positive_y_occupied() {
                    instances.push(crate::instance::Instance { position: position5, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(-90.0)), texture_offset: texture_offsets[4] });
                }
    
                if !block.negative_y_occupied() {
                    instances.push(crate::instance::Instance { position: position6, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(90.0)), texture_offset: texture_offsets[5] });
                }
    
                // instances.push(crate::instance::Instance { position: position1, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(0.0)), texture_offset: (0.0 * 0.0625, 0.0).into() });
                // instances.push(crate::instance::Instance { position: position2, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0)), texture_offset: (0.0 * 0.0625, 0.0).into() });
                // instances.push(crate::instance::Instance { position: position3, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(180.0)), texture_offset: (0.0 * 0.0625, 0.0).into() });
                // instances.push(crate::instance::Instance { position: position4, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(-90.0)), texture_offset: (0.0 * 0.0625, 0.0).into() });
                // instances.push(crate::instance::Instance { position: position5, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(-90.0)), texture_offset: (0.0 * 0.0625, 0.0).into() });
                // instances.push(crate::instance::Instance { position: position6, rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(90.0)), texture_offset: (0.0 * 0.0625, 0.0).into() });
            }
        }
        
        println!("chunk_instance_stiching: {:?}", now.elapsed());
        
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&instances.iter().map(|f| (*f).into()).collect::<Vec<crate::instance::InstanceRaw>>())
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout
            ],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                buffers: &[
                    Vertex::desc(),
                    crate::instance::InstanceRaw::desc(),
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&Block::VERTICES)
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(&Block::INDICES)
        });

        let camera_controller = crate::camera::CameraController::new(4.0);

        Self { config, device, queue, render_pipeline, size, surface, window, vertex_buffer, index_buffer, diffuse_bind_group, camera, camera_bind_group, camera_controller, camera_uniform, camera_buffer, instances, instance_buffer, depth_texture }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.draw_indexed(0..Block::INDICES.len() as u32, 0, 0..self.instances.len() as u32);
            // render_pass.draw(0..VERTICES.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) {
        self.camera_controller.process_events(event);
    }

    pub fn update(&mut self, dt: f32) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_projection(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }
}