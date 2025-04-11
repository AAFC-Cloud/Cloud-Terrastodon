use crate::app::MyApp;
use crate::loadable::Loadable;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use eframe::egui;
use eframe::egui::Checkbox;
use eframe::egui::ScrollArea;
use eframe::egui::Widget;
use eframe::egui::Window;
use eframe::egui::collapsing_header::CollapsingState;
use tracing::debug;
use tracing::info;

impl MyApp {
    pub fn draw_app(&mut self, ctx: &eframe::egui::Context) {
        let app = self;
        Window::new("Starting Points").show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let mut expando = CollapsingState::load_with_default_open(
                        ctx,
                        "subscriptions".into(),
                        app.toggle_subscriptions_expando,
                    );
                    if app.toggle_subscriptions_expando {
                        expando.toggle(&ui);
                        app.toggle_subscriptions_expando = false;
                    }
                    let is_open = expando.is_open();
                    if is_open && matches!(app.subscriptions, Loadable::NotLoaded) {
                        info!("Queueing work to fetch subscriptions");
                        LoadableWorkBuilder::new()
                            .field(|app| &mut app.subscriptions)
                            .work(async move {
                                let subs = fetch_all_subscriptions().await?;
                                // default to not-expanded
                                Ok(subs.into_iter().map(|sub| (false, sub)).collect())
                            })
                            .build()
                            .unwrap()
                            .enqueue(app);
                    }
                    expando
                        .clone()
                        .show_header(ui, |ui| match &mut app.subscriptions {
                            Loadable::Loaded(subs) => {
                                if ui
                                    .image(egui::include_image!(
                                        "../assets/10002-icon-service-Subscriptions-4x.png"
                                    ))
                                    .clicked()
                                {
                                    debug!("Clicked on subscription icon");
                                    app.toggle_subscriptions_expando = true;
                                }

                                let mut all = subs.iter().all(|(checked, _)| *checked);
                                let any = subs.iter().any(|(checked, _)| *checked);
                                let indeterminate = any && !all;
                                let elem = Checkbox::new(&mut all, "Subscriptions")
                                    .indeterminate(indeterminate)
                                    .ui(ui);
                                if elem.changed() {
                                    for (sub_checked, _) in subs.iter_mut() {
                                        *sub_checked = all;
                                    }
                                }
                            }
                            _ => {
                                if ui
                                    .image(egui::include_image!(
                                        "../assets/10002-icon-service-Subscriptions-4x.png"
                                    ))
                                    .clicked()
                                {
                                    debug!("Clicked on subscription header icon");
                                    app.toggle_subscriptions_expando = true;
                                }
                                let elem = ui.label("Subscriptions");
                                if elem.clicked() {
                                    debug!("Clicked on subscriptions header text");
                                    app.toggle_subscriptions_expando = true;
                                };
                            }
                        })
                        .body(|ui| {
                            ui.vertical(|ui| match &mut app.subscriptions {
                                Loadable::NotLoaded => {
                                    ui.label("Not loaded");
                                }
                                Loadable::Loading => {
                                    ui.label("Loading...");
                                }
                                Loadable::Loaded(subs) => {
                                    for (checked, sub) in subs.iter_mut() {
                                        ui.horizontal(|ui| {
                                            if ui.image(egui::include_image!(
                                                "../assets/10002-icon-service-Subscriptions-4x.png"
                                            )).clicked() {
                                                debug!("Clicked on subscription icon");
                                                *checked ^= true;
                                            }
                                            ui.checkbox(checked, sub.to_string());
                                        });
                                    }
                                }
                                Loadable::Failed(err) => {
                                    ui.label(&format!("Error: {}", err));
                                }
                            });
                        });
                })
            });
        });
    }
}
