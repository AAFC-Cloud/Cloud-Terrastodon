use std::collections::HashSet;
use std::path::PathBuf;
use crate::app::MyApp;
use eframe::egui::Context;
use eframe::egui::Ui;
use eframe::egui::Window;

pub fn draw_work_dirs_window(app: &mut MyApp, ctx: &Context) {
    Window::new("TF Work Dirs").show(ctx, |ui| {
        let work_dirs: HashSet<PathBuf> = app.work_dirs_config.work_dirs.clone();
        if work_dirs.is_empty() {
            ui.label("There's nothing here!");
        } else {
            ui.vertical(|ui| {
                for work_dir in work_dirs {
                    add_work_dir_row(ui, app, &work_dir);
                }
            });
        }
    });
}

fn add_work_dir_row(ui: &mut Ui, app: &mut MyApp, work_dir: &PathBuf) {
    ui.horizontal(|ui| {
        ui.label(work_dir.display().to_string());
        if ui.button("-").clicked() {
            app.work_dirs_config.work_dirs.remove(work_dir);
            // will be auto-saved later
        }
    });
}
