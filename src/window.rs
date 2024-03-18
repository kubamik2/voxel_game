use winit::{
    event::*, event_loop::{ControlFlow, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder
};

pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut window = WindowBuilder::new()
    .with_inner_size(winit::dpi::PhysicalSize::new(1000, 1000))
    .build(&event_loop).unwrap();

    window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
    window.set_cursor_visible(false);
    window.set_outer_position(winit::dpi::LogicalPosition::new(900.0, 0.0));
    let mut state = pollster::block_on(crate::state::State::new(window));
    let mut last_render_time = std::time::Duration::ZERO;

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        match event {
            Event::WindowEvent { window_id, event } if window_id == state.window.id() => {
                state.input(&event);
                match event {
                    WindowEvent::CloseRequested 
                    | WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::Escape), state: ElementState::Pressed, .. }, ..} => {
                        elwt.exit();
                    },
                    WindowEvent::RedrawRequested if window_id == state.window.id() => {
                        let now = std::time::Instant::now();
                        state.update(last_render_time.as_secs_f32());
                        let update_time = now.elapsed();
                        
                        state.render();
                        println!("render_time: {:?}", last_render_time);
                        last_render_time = now.elapsed();
                    }
                    _ => ()
                }
            },
            Event::NewEvents(StartCause::Poll) => {
                state.window.request_redraw();
            },
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta } = event {
                    state.world.camera_controller.mouse_move(delta.0 as f32, delta.1 as f32, &mut state.world.camera);
                    state.window.set_cursor_position(winit::dpi::LogicalPosition::new(0.0, 0.0));
                }
            }
            _ => ()
        }
    }).unwrap();
}