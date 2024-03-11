use cgmath::{Basis2, Basis3, Matrix4, Point3, Quaternion, Vector3, Vector2, InnerSpace, Rotation, Rotation2, Rotation3, Euler, Deg, Rad};
const PITCH_LIMIT: f32 = std::f32::consts::PI / 2.0 - 0.0001;

pub struct Camera {
    pub eye: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub direction: Vector3<f32>,
}

impl Camera {
    pub fn default(width: u32, height: u32) -> Self {
        Self {
            eye: Point3::new(0.0, 0.0, 3.0),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            up: cgmath::Vector3::unit_y(),
            aspect: width as f32 / height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            direction: Vector3::new(1.0, 0.0, 0.0)
        }
    }

    pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    );

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_to_rh(self.eye, self.direction, self.up);
        let projection = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        Self::OPENGL_TO_WGPU_MATRIX * projection * view
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct CameraUniform {
    pub view_projection: [[f32; 4]; 4]
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self { view_projection: cgmath::Matrix4::identity().into() }
    }

    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().into();
    }
}

#[derive(Default)]
pub struct Controls {
    pub forward_pressed: bool,
    pub backward_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
}

pub struct CameraController {
    pub speed: f32,
    pub controls: Controls
}
use winit::event::*;
impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            controls: Controls::default()
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                let pressed = input.state == ElementState::Pressed;
                if let Some(virtual_keycode) = input.virtual_keycode {
                    match virtual_keycode {
                        VirtualKeyCode::W => { self.controls.forward_pressed = pressed },
                        VirtualKeyCode::S => { self.controls.backward_pressed = pressed },
                        VirtualKeyCode::A => { self.controls.left_pressed = pressed },
                        VirtualKeyCode::D => { self.controls.right_pressed = pressed },
                        VirtualKeyCode::Space => { self.controls.up_pressed = pressed },
                        VirtualKeyCode::LShift => { self.controls.down_pressed = pressed },
                        _ => ()
                    }
                }
            },
            _ => ()
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, dt: f32) {
        let forward = Vector3::new(camera.direction.x, 0.0, camera.direction.z).normalize();
        let right = forward.cross(camera.up);

        if self.controls.forward_pressed {
            camera.eye += self.speed * dt * forward;
        }

        if self.controls.backward_pressed {
            camera.eye -= self.speed * dt * forward;
        }
        
        if self.controls.right_pressed {
            camera.eye += self.speed * dt * right;
        }

        if self.controls.left_pressed {
            camera.eye -= self.speed * dt * right;
        }

        if self.controls.up_pressed {
            camera.eye.y += self.speed * dt
        }

        if self.controls.down_pressed {
            camera.eye.y -= self.speed * dt
        }
    }

    pub fn mouse_move(&mut self, delta_x: f32, delta_y: f32, camera: &mut Camera) {
        camera.yaw += Deg(delta_x / 8.0).into();
        camera.pitch -= Deg(delta_y / 8.0).into();
        camera.pitch.0 = camera.pitch.0.clamp(-PITCH_LIMIT, PITCH_LIMIT);

        let (sin_pitch, cos_pitch) = camera.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = camera.yaw.0.sin_cos();

        let direction = Vector3::new(
            cos_pitch * cos_yaw,
            sin_pitch,
            cos_pitch * sin_yaw
        ).normalize();

        camera.direction = direction;
    }
}