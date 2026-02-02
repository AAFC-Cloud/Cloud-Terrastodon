use crate::app::MyApp;
use crate::widgets::devops_project_list_expando::draw_devops_project_list_expando;
use crate::widgets::subscription_checkbox_list_expando::draw_subscription_list_expando;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Window;

pub fn draw_starting_points_window(app: &mut MyApp, ctx: &Context) {
    // Keep backwards compatible floating window behavior
    let window_id = Id::new("Starting Points");
    Window::new("Starting Points")
        .id(window_id)
        .default_size(app.egui_config.starting_points_window.size())
        .collapsible(false)
        .show(ctx, |ui| {
            starting_points_ui(app, ui);
        });

    // https://github.com/emilk/egui/issues/493#issuecomment-1859328201
    if let Some(window_area) = ctx.memory(|mem| mem.area_rect(window_id)) {
        app.egui_config.starting_points_window = window_area;
    }
}

/// Tile-friendly UI for the Starting Points pane
pub fn starting_points_ui(app: &mut MyApp, ui: &mut eframe::egui::Ui) {
    use eframe::egui::ScrollArea;
    ui.vertical_centered(|ui| {
        ScrollArea::both().show(ui, |ui| {
            draw_subscription_list_expando(app, ui);
            draw_devops_project_list_expando(app, ui);
        })
    });
}
