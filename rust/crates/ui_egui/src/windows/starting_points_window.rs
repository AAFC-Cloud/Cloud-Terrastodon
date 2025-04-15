use crate::app::MyApp;
use crate::widgets::devops_project_list_expando::draw_devops_project_list_expando;
use crate::widgets::subscription_checkbox_list_expando::draw_subscription_list_expando;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Window;

pub fn draw_starting_points_window(app: &mut MyApp, ctx: &Context) {
    let window_id = Id::new("Starting Points");
    Window::new("Starting Points")
        .id(window_id)
        .default_size(app.egui_config.starting_points_window.size())
        .collapsible(false)
        .show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    draw_subscription_list_expando(app, ctx, ui);
                    draw_devops_project_list_expando(app, ctx, ui);
                })
            });
        });

    // https://github.com/emilk/egui/issues/493#issuecomment-1859328201
    if let Some(window_area) = ctx.memory(|mem| mem.area_rect(window_id)) {
        app.egui_config.starting_points_window = window_area;
    }
}
