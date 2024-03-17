
pub struct Gui {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub render_time: std::time::Duration,
    pub update_time: std::time::Duration,
    pub blocks: usize,
    pub vertices: usize,
    pub memory_usage: usize,
}

impl Gui {
    pub fn ui(&self, ctx: &egui::Context) {
        egui::Window::new("test")
        .title_bar(false)
        .show(ctx, |ui| {
            ui.label(format!("x: {:.2}, y: {:.2}, z: {:.2}", self.position[0], self.position[1], self.position[2]));
            ui.label(format!("render_time: {:.2?}", self.render_time));
            ui.label(format!("update_time: {:.2?}", self.update_time));
            ui.label(format!("fps: {:.2}", 1.0 / self.render_time.as_secs_f32()));
            ui.label(format!("blocks: {}", self.blocks));
            ui.label(format!("vertices: {}", self.vertices));
            ui.label(format!("memory_usage: {:.2} MiB", self.memory_usage as f32 / 1_048_576.0))
        });
    }
}
