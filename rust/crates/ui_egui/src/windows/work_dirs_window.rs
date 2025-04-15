use crate::app::MyApp;
use eframe::egui::Context;
use eframe::egui::Window;

pub fn draw_work_dirs_window(app: &mut MyApp, ctx: &Context) {
    Window::new("TF Work Dirs").show(ctx, |ui| {
        let work_dirs = &app.work_dirs_config.work_dirs;
        if work_dirs.is_empty() {
            ui.label("There's nothing here!");
        } else {
            ui.vertical(|ui| {
                for work_dir in work_dirs {
                    ui.label(work_dir.display().to_string());
                }
            });
        }
    });
}
