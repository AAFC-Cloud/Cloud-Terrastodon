use super::subscription_checkbox_list::draw_subscription_checkbox_list;
use crate::app::MyApp;
use crate::icons::SUBSCRIPTION_ICON;
use crate::loadable::Loadable;
use crate::workers::load_subscriptions::load_subscriptions;
use eframe::egui::Checkbox;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Ui;
use eframe::egui::Widget;
use eframe::egui::collapsing_header::CollapsingState;

pub fn draw_subscription_list_expando(app: &mut MyApp, ctx: &Context, ui: &mut Ui) {
    let mut expando = CollapsingState::load_with_default_open(ctx, "subscriptions".into(), false);
    let toggle_key = expando.id();
    if app.toggle_intents.remove(&toggle_key) {
        expando.toggle(ui);
    }
    if expando.is_open() && matches!(app.subscriptions, Loadable::NotLoaded) {
        load_subscriptions(app);
    }
    expando
        .clone()
        .show_header(ui, |ui| draw_header(app, ui, toggle_key))
        .body(|ui| draw_subscription_checkbox_list(app, ctx, ui));
}

fn draw_header(app: &mut MyApp, ui: &mut Ui, toggle_key: Id) {
    match &app.subscriptions {
        Loadable::Loaded(subs) => {
            let subs = subs.clone();
            if ui.image(SUBSCRIPTION_ICON).clicked() {
                app.toggle_intents.insert(toggle_key);
            }

            let mut all = subs.iter().all(|sub| *app.checkbox_for(sub.id));
            let any = subs.iter().any(|sub| *app.checkbox_for(sub.id));
            let indeterminate = any && !all;
            let elem = Checkbox::new(&mut all, "Subscriptions")
                .indeterminate(indeterminate)
                .ui(ui);
            if elem.changed() {
                for sub in subs.iter() {
                    *app.checkbox_for(sub.id) = all
                }
            }
        }
        _ => {
            if ui.image(SUBSCRIPTION_ICON).clicked() {
                app.toggle_intents.insert(toggle_key);
            }
            let elem = ui.label("Subscriptions");
            if elem.clicked() {
                app.toggle_intents.insert(toggle_key);
            };
        }
    }
}
