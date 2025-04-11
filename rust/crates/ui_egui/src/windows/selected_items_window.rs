use eframe::egui;
use eframe::egui::Context;
use eframe::egui::ScrollArea;
use eframe::egui::Window;
use tracing::debug;

use crate::app::MyApp;
use crate::loadable::Loadable;

pub fn draw_selected_items_window(app: &mut MyApp, ctx: &Context) {
    Window::new("Selected Items").show(ctx, |ui| {
        ScrollArea::both().show(ui, |ui| {
            if let Loadable::Loaded(subscriptions) = &app.subscriptions {
                for subscription in subscriptions.clone().iter() {
                    let checked = app.checkbox_for(&subscription.id);
                    if !*checked {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        if ui
                            .image(egui::include_image!(
                                "../../assets/10002-icon-service-Subscriptions-4x.png"
                            ))
                            .clicked()
                        {
                            debug!("Clicked on subscription icon");
                            *checked ^= true;
                        }
                        ui.checkbox(checked, subscription.to_string());
                    });
                }
            }
            if let Loadable::Loaded(projects) = &app.azure_devops_projects {
                for project in projects.clone().iter() {
                    let checked = app.checkbox_for(&project.id);
                    if !*checked {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        if ui
                            .image(egui::include_image!(
                                "../../assets/10261-icon-service-Azure-DevOps-4x.png"
                            ))
                            .clicked()
                        {
                            debug!("Clicked on Azure DevOps Project icon");
                            *checked ^= true;
                        }
                        ui.checkbox(checked, project.name.to_string());
                    });
                }
            }
        });
    });
}
