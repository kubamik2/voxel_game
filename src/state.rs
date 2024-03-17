use std::borrow::Borrow;

use cgmath::{Point3, Rotation3};
use winit::{raw_window_handle::HasWindowHandle, window::Window};
use crate::{block::{Bitmap, Block, Material}, chunk::{Chunk, World, WORLD_SIZE}, egui_renderer::EguiRenderer, vertex::{Vertex, VertexPacked}};
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
    pub depth_texture: crate::texture::Texture,
    pub vertex_array: Vec<VertexPacked>,
    pub index_array: Vec<u32>,
    pub egui_renderer: EguiRenderer,
    pub blocks: usize,
    pub vertices: usize,
    pub memory_usage: usize,
    pub shader: wgpu::ShaderModule,
    pub render_pipeline_layout: wgpu::PipelineLayout,
}

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

        let limits = wgpu::Limits {
            max_buffer_size: 1024 * 1024 * 1024,
            ..Default::default()
        };

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default() | wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::MULTI_DRAW_INDIRECT,
            limits
        }, None).await.unwrap();

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

        let mut world = World::new();
        // let mut chunks = vec![];
        // use rand::Rng;
        // let mut rng = rand::thread_rng();
        // for ix in 0..19 {
        //     let ix = ix as f32;
        //     for iz in 0..19 {
        //         let iz = iz as f32;

        //         let mut blocks = vec![];
        //         for y in 0..256 {
        //             for z in 0..16 {
        //                 for x in 0..16 {
        //                     let material = if y < 129 { Material::Cobblestone } else { Material::Air };
        //                     blocks.push(Block { position: Point3::new(x as f32 + ix * 16.0, y as f32, z as f32 + iz * 16.0), bitmap: Bitmap(0), material });
        //                 }
        //             }
        //         }
    
        //         chunks.push(Chunk { blocks });
        //     }
        // }

        let mut vertex_array = vec![];
        let mut index_array = vec![];
        let mut index_offset = 0;

        for i in 0..WORLD_SIZE {
            for j in 0..WORLD_SIZE {
                world.bake_chunk((i, j))
            }
        }
        let mut blocks = 0;
        for chunk in world.chunks.iter_mut() {
            blocks += chunk.blocks.iter().filter(|p| p.material != Material::Air).count();
            index_offset = chunk.push_vertex_data(&mut vertex_array, &mut index_array, index_offset);
        }
        let memory_usage = vertex_array.len() * std::mem::size_of::<VertexPacked>() + index_array.len() * std::mem::size_of::<u32>();
        let vertices = vertex_array.len();

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
                    VertexPacked::desc(),
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
            contents: bytemuck::cast_slice(vertex_array.as_slice())
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            usage: wgpu::BufferUsages::INDEX,
            contents: bytemuck::cast_slice(index_array.as_slice())
        });


        let camera_controller = crate::camera::CameraController::new(1.0);

        let egui_renderer = EguiRenderer::new(&window, &config, &device);

        Self { config, device, queue, render_pipeline, size, surface, window, vertex_buffer, index_buffer, diffuse_bind_group, camera, camera_bind_group, camera_controller, camera_uniform, camera_buffer, depth_texture, vertex_array, index_array, egui_renderer, blocks, vertices, memory_usage, shader, render_pipeline_layout }
    }

    pub fn render(&mut self, render_time: std::time::Duration, update_time: std::time::Duration) -> Result<(), wgpu::SurfaceError> {
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
            //render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

           // render_pass.draw_indexed(0..Block::INDICES.len() as u32, 0, 0..self.instances.len() as u32);
            // render_pass.draw(0..self.vertex_array.len() as u32, 0..1);
            render_pass.draw_indexed(0..self.index_array.len() as u32, 0, 0..1);
            // render_pass.draw(0..VERTICES.len() as u32, 0..1);
        }

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32
        };

        self.egui_renderer.draw(&self.device, &self.queue, &mut encoder, &self.window, &view, screen_descriptor, |ctx| {
            let gui = crate::gui::Gui { position: self.camera.eye.into(), direction: self.camera.direction.into(), render_time, update_time, blocks: self.blocks, vertices: self.vertices, memory_usage: self.memory_usage };
            gui.ui(ctx);
        });

        self.queue.submit(std::iter::once(encoder.finish()));
        self.window.pre_present_notify();
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

        let polygon_mode = if self.camera_controller.controls.f1_toggled { wgpu::PolygonMode::Line } else { wgpu::PolygonMode::Fill };

        self.render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&self.render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.shader,
                buffers: &[
                    VertexPacked::desc(),
                ],
                entry_point: "vs_main"
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.format,
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
                polygon_mode,
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
    }
}