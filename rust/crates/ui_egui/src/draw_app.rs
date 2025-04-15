use crate::app::MyApp;
use crate::windows::selected_items_window::draw_selected_items_window;
use crate::windows::starting_points_window::draw_starting_points_window;
use crate::windows::work_dirs_window::draw_work_dirs_window;
impl MyApp {
    pub fn draw_app(&mut self, ctx: &eframe::egui::Context) {
        let app = self;
        draw_starting_points_window(app, ctx);
        draw_selected_items_window(app, ctx);
        draw_work_dirs_window(app, ctx);
    }
}
