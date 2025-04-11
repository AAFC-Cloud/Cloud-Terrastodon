use crate::app::MyApp;
use crate::windows::selected_items_window::draw_selected_items_window;
use crate::windows::subscriptions_window::draw_subscriptions_window;
impl MyApp {
    pub fn draw_app(&mut self, ctx: &eframe::egui::Context) {
        let app = self;
        draw_subscriptions_window(app, ctx);
        draw_selected_items_window(app, ctx);
    }
}
