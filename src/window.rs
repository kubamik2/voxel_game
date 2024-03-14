use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder
};

pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut window = WindowBuilder::new()
    .with_inner_size(winit::dpi::PhysicalSize::new(1000, 1000))
    .build(&event_loop).unwrap();

    window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
    window.set_cursor_visible(false);
    let mut state = pollster::block_on(crate::state::State::new(window));
    let mut last_render_time = std::time::Duration::ZERO;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { window_id, event } if window_id == state.window.id() => {
                state.input(&event);
                match event {
                    WindowEvent::CloseRequested 
                    | WindowEvent::KeyboardInput { input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. }, .. } => {
                        control_flow.set_exit();
                    },
                    _ => ()
                }
            },
            Event::RedrawRequested(window_id) if window_id == state.window.id() => {
                let now = std::time::Instant::now();
                state.update(last_render_time.as_secs_f32());
                let update_time = now.elapsed();
            
                state.render();
                let render_time = now.elapsed() - update_time;
                //println!("update_time: {:?}\nrender_time: {:?}\n", update_time, render_time);
                last_render_time = now.elapsed();
            },
            Event::MainEventsCleared => {
                state.window.request_redraw();
            },
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta } = event {
                    state.camera_controller.mouse_move(delta.0 as f32, delta.1 as f32, &mut state.camera);
                }
            }
            _ => ()
        }
    });
}