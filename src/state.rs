use cgmath::{Point2, Point3};
use winit::window::Window;
use crate::chunk::World;

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Window,
    pub world: World
}

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)
        }).await.unwrap();

        let limits = wgpu::Limits {
            max_buffer_size: u32::MAX as u64,
            ..Default::default()
        };

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default() | wgpu::Features::MULTI_DRAW_INDIRECT | wgpu::Features::INDIRECT_FIRST_INSTANCE,
            limits
        }, None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|p| p.is_srgb()).unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };

        surface.configure(&device, &config);

        let world = World::new(&window, &device, &config, &queue);

        Self { window, device, config, queue, size, surface, world }
    }

    pub fn render(&mut self, render_time: std::time::Duration, update_time: std::time::Duration) -> Result<(), wgpu::SurfaceError> {
        self.world.render(&self.device, &self.queue, &self.config,&self.surface, &self.window, render_time, update_time);

        Ok(())
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) {
        self.world.camera_controller.process_events(event);
    }

    pub fn update(&mut self, dt: f32) {
        self.world.camera_controller.update_camera(&mut self.world.camera, dt);
        self.world.camera_uniform.update_view_projection(&self.world.camera);
        self.queue.write_buffer(&self.world.camera_buffer, 0, bytemuck::cast_slice(&[self.world.camera_uniform]));
    }
}