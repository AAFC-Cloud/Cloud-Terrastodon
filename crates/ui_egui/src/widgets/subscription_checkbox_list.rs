use super::subscription_checkbox::draw_subscription_checkbox;
use crate::app::MyApp;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Ui;

pub fn draw_subscription_checkbox_list(app: &mut MyApp, ui: &mut Ui) {
    ui.vertical(|ui| match &app.subscriptions {
        Loadable::NotLoaded => {
            ui.label("Not loaded");
        }
        Loadable::Loading { .. } => {
            ui.label("Loading...");
        }
        Loadable::Loaded { value: subs, .. } => {
            let subs = subs.clone();
            for subscription in subs.iter() {
                draw_subscription_checkbox(app, ui, subscription);
            }
        }
        Loadable::Failed { error: err, .. } => {
            ui.label(format!("Error: {err}"));
        }
    });
}
