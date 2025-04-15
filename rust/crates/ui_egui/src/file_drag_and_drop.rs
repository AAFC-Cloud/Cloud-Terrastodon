use crate::app::MyApp;
use eframe::egui;
use eframe::egui::Context;
use tracing::info;

pub fn ui_file_drag_and_drop(app: &mut MyApp, ctx: &Context) {
    use egui::Align2;
    use egui::Color32;
    use egui::Id;
    use egui::LayerId;
    use egui::Order;
    use egui::TextStyle;
    use std::fmt::Write as _;

    // Preview hovering files:
    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }

    // Collect dropped files:
    let tx = app.tx.clone();
    let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
    if !dropped_files.is_empty() {
        info!("Dropped files: {:?}", dropped_files);
        for file in dropped_files {
            if let Some(path) = file.path {
                app.work_dirs_config.work_dirs.insert(path);
            }
        }
    }
}
