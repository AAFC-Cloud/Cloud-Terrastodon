use eframe::egui::Context;
use eframe::egui::Window;

use crate::app::MyApp;

pub fn draw_work_dirs_window(app: &mut MyApp, ctx: &Context) {
    Window::new("TF Work Dirs").show(ctx, |ui| {
        
        ui.label("There's nothing here!");
    });
}
