use crate::app::MyApp;
use crate::loadable::Loadable;
use crate::workers::load_resource_groups::load_resource_groups;
use cloud_terrastodon_core_azure::prelude::Subscription;
use eframe::egui;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Ui;
use eframe::egui::collapsing_header::CollapsingState;
use tracing::debug;

pub fn draw_subscription_checkbox(
    app: &mut MyApp,
    ctx: &Context,
    ui: &mut Ui,
    subscription: &Subscription,
) {
    let mut expando =
        CollapsingState::load_with_default_open(ctx, Id::new(subscription.id.clone()), false);
    let toggle_key = expando.id();
    if app.toggle_intents.remove(&toggle_key) {
        expando.toggle(&ui);
    }
    let is_open = expando.is_open();
    if is_open && matches!(app.resource_groups, Loadable::NotLoaded) {
        load_resource_groups(app);
    }
    expando
        .clone()
        .show_header(ui, |ui| {
            ui.horizontal(|ui| {
                let resource_group_count = app
                    .resource_groups
                    .as_loaded()
                    .and_then(|resource_groups| resource_groups.get(&subscription.id))
                    .map(|list| list.len());
                let label = match resource_group_count {
                    Some(resource_group_count) => {
                        format!("{} ({})", subscription, resource_group_count)
                    }
                    None => format!("{}", subscription),
                };

                let checked = app.checkbox_for(&subscription.id);
                if ui
                    .image(egui::include_image!(
                        "../../assets/10002-icon-service-Subscriptions-4x.png"
                    ))
                    .clicked()
                {
                    debug!("Clicked on subscription icon");
                    *checked ^= true;
                }

                ui.checkbox(checked, label);
            });
        })
        .body(|ui| match &app.resource_groups {
            Loadable::NotLoaded => {
                ui.label("Not loaded");
            }
            Loadable::Loading => {
                ui.label("Loading...");
            }
            Loadable::Loaded(resource_groups) => {
                let resource_groups = resource_groups.clone();
                let resource_groups = resource_groups.get(&subscription.id);
                ui.vertical(|ui| match resource_groups {
                    None => {
                        ui.label("This subscription has no resource groups");
                    }
                    Some(resource_groups) => {
                        for resource_group in resource_groups {
                            ui.label(format!("{}", resource_group.name));
                        }
                    }
                });
            }
            Loadable::Failed(err) => {
                ui.label(&format!("Error: {}", err));
            }
        });
}
