use crate::app::MyApp;
use crate::widgets::devops_project_list_expando::draw_devops_project_list_expando;
use crate::widgets::subscription_list_expando::draw_subscription_list_expando;
use eframe::egui::Context;
use eframe::egui::ScrollArea;
use eframe::egui::Window;

pub fn draw_starting_points_window(app: &mut MyApp, ctx: &Context) {
    Window::new("Starting Points").show(ctx, |ui| {
        ScrollArea::both().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                draw_subscription_list_expando(app, ctx, ui);
                draw_devops_project_list_expando(app, ctx, ui);
            })
        });
    });
}
