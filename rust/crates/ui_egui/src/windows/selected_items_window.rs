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
            if let Loadable::Loaded(subscriptions) = &mut app.subscriptions {
                for (checked, sub) in subscriptions.iter_mut() {
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
                        ui.checkbox(checked, sub.to_string());
                    });
                }
            }
        });
    });
}
