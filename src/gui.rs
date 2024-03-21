use crate::chunk::{CHUNK_HEIGHT, CHUNK_SIZE, RENDER_DISTANCE};


pub struct Gui {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub render_time: std::time::Duration,
    pub update_time: std::time::Duration,
}

impl Gui {
    pub fn ui(&self, ctx: &egui::Context) {
        egui::Window::new("test")
        .title_bar(false)
        .show(ctx, |ui| {
            ui.label(format!("x: {:.2}, y: {:.2}, z: {:.2}", self.position[0], self.position[1], self.position[2]));
            ui.label(format!("dx: {:.2}, dy: {:.2}, dz: {:.2}", self.direction[0], self.direction[1], self.direction[2]));
            ui.label(format!("render_time: {:.2?}", self.render_time));
            ui.label(format!("update_time: {:.2?}", self.update_time));
            ui.label(format!("fps: {:.2}", 1.0 / self.render_time.as_secs_f32()));
            ui.label(format!("blocks: {}", CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT * RENDER_DISTANCE * RENDER_DISTANCE));
        });
    }
}
