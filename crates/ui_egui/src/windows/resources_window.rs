use crate::app::MyApp;
use crate::loadable::Loadable;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_azure::prelude::Resource;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use eframe::egui::Ui;
use egui_extras::Column;
use egui_extras::TableBuilder;
use std::sync::Arc;

pub fn resources_ui(app: &mut MyApp, ui: &mut Ui) {
    match &app.resources {
        Loadable::NotLoaded => {
            // Automatically enqueue a background fetch when the Resources pane is shown.
            let work = LoadableWorkBuilder::new()
                .description("Fetch resources")
                .setter(|app, l| app.resources = l)
                .work(async { Ok(Arc::new(fetch_all_resources().await?)) })
                .build()
                .expect("building work");
            work.enqueue(app);
            ui.label("Loading resources...");
        }
        Loadable::Loading => {
            ui.label("Loading resources...");
        }
        Loadable::Failed(err) => {
            ui.label(format!("Failed to load resources: {:#?}", err));
        }
        Loadable::Loaded(resources_rc) => {
            let resources: &Vec<Resource> = resources_rc;

            let available_height = ui.available_height();
            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .column(Column::auto()) // Name
                .column(Column::remainder().at_least(160.0).clip(true)) // Id
                .column(Column::auto()) // Kind
                .column(Column::remainder()) // Display Name
                .min_scrolled_height(0.0)
                .max_scroll_height(available_height);

            table = table.sense(eframe::egui::Sense::click());

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Id");
                    });
                    header.col(|ui| {
                        ui.strong("Kind");
                    });
                    header.col(|ui| {
                        ui.strong("Display Name");
                    });
                })
                .body(|body| {
                    body.rows(20.0, resources.len(), |mut row| {
                        let r = &resources[row.index()];
                        row.col(|ui| {
                            ui.label(&r.name);
                        });
                        row.col(|ui| {
                            ui.monospace(r.id.expanded_form());
                        });
                        row.col(|ui| {
                            ui.label(r.kind.as_ref());
                        });
                        row.col(|ui| {
                            if let Some(ref d) = r.display_name {
                                ui.label(d);
                            } else {
                                ui.label("");
                            }
                        });
                    });
                });
        }
    }
}
