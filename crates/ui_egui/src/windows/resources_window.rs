use crate::app::MyApp;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use eframe::egui::Ui;
use egui_extras::Column;
use egui_extras::TableBuilder;
use std::sync::Arc;
use std::time::Instant;

pub fn resources_ui(app: &mut MyApp, ui: &mut Ui) {
    match &app.resources {
        Loadable::NotLoaded => {
            let tenant_id = app.tenant_id;
            let auth_context = app.auth_context.clone();
            let executor = app.executor.clone();
            let cancellation = executor.cancellation().clone();
            let started_at = Instant::now();
            // Automatically enqueue a background fetch when the Resources pane is shown.
            let work = LoadableWorkBuilder::<MyApp, _>::new()
                .description("Fetch resources")
                .setter(|app, l| app.resources = l)
                .work(async move |scope| {
                    let executor = executor.with_cancellation_scope(scope);
                    let auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
                    let executor = auth_context.auth_context.bind_executor(&executor)?;
                    Ok(Arc::new(
                        fetch_all_resources(&auth_context, &executor).await?,
                    ))
                })
                .build()
                .and_then(|work| {
                    work.enqueue(
                        &app.work_tracker.clone(),
                        app.tx.clone(),
                        app,
                        &cancellation,
                    )
                });
            match work {
                Ok(()) => {
                    ui.label("Loading resources...");
                }
                Err(error) => {
                    ui.label(format!("Failed to prepare resource loading: {error:#}"));
                    app.resources = Loadable::Failed {
                        error,
                        started_at,
                        finished_at: Instant::now(),
                    };
                }
            }
        }
        Loadable::Loading { .. } => {
            ui.label("Loading resources...");
        }
        Loadable::Failed { error: err, .. } => {
            ui.label(format!("Failed to load resources: {:#?}", err));
        }
        Loadable::Loaded {
            value: resources_rc,
            ..
        } => {
            let resources: &Vec<Arc<Resource>> = resources_rc;

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
                            if let Some(d) = r.display_name() {
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
