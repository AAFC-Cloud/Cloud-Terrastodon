use crate::app::MyApp;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Align2;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::LayerId;
use eframe::egui::Order;
use eframe::egui::TextStyle;

pub fn ui_file_drag_and_drop(app: &mut MyApp, ctx: &Context) {
    if !ctx.input(|input| input.raw.hovered_files.is_empty()) {
        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
        let rect = ctx.content_rect();
        painter.rect_filled(rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Drop to open files or directories",
            TextStyle::Heading.resolve(&ctx.global_style()),
            Color32::WHITE,
        );
    }

    let paths: Vec<_> = ctx.input(|input| {
        input
            .raw
            .dropped_files
            .iter()
            .map(|file| file.path().to_owned())
            .collect()
    });
    if paths.is_empty() {
        return;
    }
    for path in paths {
        app.work_dirs_config.work_dirs.insert(path.clone());
        // Keep every path when a user drops several items together.
        let target = PortalTabTarget::LocalPath(path);
        let new_visit = !app.tabs.contains(&target);
        app.tabs.open_kept(target);
        if new_visit {
            app.record_portal_visit();
        }
    }
    app.azure.resource_search.clear();
    app.retain_open_tab_data();
}
