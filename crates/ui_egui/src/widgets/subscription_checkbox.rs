use super::resource_group_checkbox::draw_resource_group_checkbox;
use crate::app::MyApp;
use crate::icons::SUBSCRIPTION_ICON;
use crate::loadable::Loadable;
use crate::workers::load_resource_groups::load_resource_groups;
use cloud_terrastodon_azure::prelude::Subscription;
use eframe::egui::Id;
use eframe::egui::Ui;
use eframe::egui::collapsing_header::CollapsingState;
use tracing::debug;

pub fn draw_subscription_checkbox(app: &mut MyApp, ui: &mut Ui, subscription: &Subscription) {
    let mut expando =
        CollapsingState::load_with_default_open(ui.ctx(), Id::new(subscription.id), false);
    let toggle_key = expando.id();
    if app.toggle_intents.remove(&toggle_key) {
        expando.toggle(ui);
    }
    let is_open = expando.is_open();
    if is_open && matches!(app.resource_groups, Loadable::NotLoaded) {
        load_resource_groups(app);
    }
    expando
        .clone()
        .show_header(ui, |ui| {
            draw_header(app, subscription, ui);
        })
        .body(|ui| draw_body(app, subscription, ui));
}

fn draw_header(app: &mut MyApp, subscription: &Subscription, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let resource_group_count = app
            .resource_groups
            .as_loaded()
            .map(|resource_groups| resource_groups.get_for_subscription(&subscription.id))
            .map(|list| list.len());
        let label = match resource_group_count {
            Some(resource_group_count) => {
                format!("{subscription} ({resource_group_count})")
            }
            None => format!("{subscription}"),
        };

        let checked = app.checkbox_for(subscription.id);
        if ui.image(SUBSCRIPTION_ICON).clicked() {
            debug!("Clicked on subscription icon");
            *checked ^= true;
        }

        ui.checkbox(checked, label);
    });
}

fn draw_body(app: &mut MyApp, subscription: &Subscription, ui: &mut Ui) {
    match &app.resource_groups {
        Loadable::NotLoaded => {
            ui.label("Not loaded");
        }
        Loadable::Loading => {
            ui.label("Loading...");
        }
        Loadable::Loaded(resource_groups) => {
            let resource_groups = resource_groups.clone();
            let resource_groups = resource_groups.get_for_subscription(&subscription.id);
            ui.vertical(|ui| {
                if resource_groups.is_empty() {
                    ui.label("This subscription has no resource groups");
                } else {
                    for resource_group in resource_groups {
                        draw_resource_group_checkbox(app, ui, resource_group);
                    }
                }
            });
        }
        Loadable::Failed(err) => {
            ui.label(format!("Error: {err}"));
        }
    }
}
