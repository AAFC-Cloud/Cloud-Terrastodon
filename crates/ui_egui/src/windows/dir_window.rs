use std::path::PathBuf;

use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Ui;
use eframe::egui::Window;

use crate::app::MyApp;

pub fn ui_dir_windows(app: &mut MyApp, ctx: &Context) {
    for (dir, rect) in app.egui_config.open_dirs.clone().into_iter() {
        let title = format!("{}", dir.display());
        let window_id = Id::new(&title);
        let mut open = true;
        Window::new(title)
            .id(window_id)
            .default_rect(rect)
            .open(&mut open)
            .collapsible(true)
            .show(ctx, |ui| {
                ui_dir_window_body(app, ctx, ui, &dir);
            });
        if !open {
            app.egui_config.open_dirs.remove(&dir).unwrap();
        } else if let Some(window_area) = ctx.memory(|mem| mem.area_rect(window_id)) {
            app.egui_config.open_dirs.insert(dir, window_area);
        }
    }
}

pub fn ui_dir_window_body(_app: &mut MyApp, _ctx: &Context, ui: &mut Ui, _dir: &PathBuf) {
    ui.label("bruh");
}
