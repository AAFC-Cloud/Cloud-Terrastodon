use super::subscription_checkbox::draw_subscription_checkbox;
use crate::app::MyApp;
use crate::loadable::Loadable;
use eframe::egui::Context;
use eframe::egui::Ui;

pub fn draw_subscription_checkbox_list(app: &mut MyApp, ctx: &Context, ui: &mut Ui) {
    ui.vertical(|ui| match &app.subscriptions {
        Loadable::NotLoaded => {
            ui.label("Not loaded");
        }
        Loadable::Loading => {
            ui.label("Loading...");
        }
        Loadable::Loaded(subs) => {
            let subs = subs.clone();
            for subscription in subs.iter() {
                draw_subscription_checkbox(app, ctx, ui, subscription);
            }
        }
        Loadable::Failed(err) => {
            ui.label(format!("Error: {err}"));
        }
    });
}
