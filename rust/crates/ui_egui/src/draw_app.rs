use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use eframe::egui::Checkbox;
use eframe::egui::ScrollArea;
use eframe::egui::Widget;
use eframe::egui::Window;
use eframe::egui::collapsing_header::CollapsingState;
use tracing::debug;

use crate::app::MyApp;
use crate::loadable::Loadable;
use crate::state_mutator::StateMutator;
use crate::work::Work;

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
                        #[derive(Debug)]
                        struct UpdateSubscriptionsSuccess(Vec<Subscription>);
                        impl StateMutator for UpdateSubscriptionsSuccess {
                            fn mutate_state(self: Box<Self>, state: &mut MyApp) {
                                state.subscriptions = Loadable::Loaded(
                                    self.0.into_iter().map(|x| (false, x)).collect(),
                                );
                            }
                        }
                        #[derive(Debug)]
                        struct UpdateSubscriptionsFailure(eyre::ErrReport);
                        impl StateMutator for UpdateSubscriptionsFailure {
                            fn mutate_state(self: Box<Self>, state: &mut MyApp) {
                                state.subscriptions = Loadable::Failed(self.0)
                            }
                        }
                        Work {
                            on_enqueue: |app| app.subscriptions = Loadable::Loading,
                            on_work: async move {
                                let subscriptions = fetch_all_subscriptions().await?;
                                Ok(UpdateSubscriptionsSuccess(subscriptions))
                            },
                            on_failure: UpdateSubscriptionsFailure,
                        }
                        .enqueue(app);
                        crate::work::FieldUpdaterWorkBuilder::new()
                            .field(|app| &mut app.subscriptions)
                            .build();
                    }
                    expando
                        .clone()
                        .show_header(ui, |ui| match &mut app.subscriptions {
                            Loadable::Loaded(subs) => {
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
                                let elem = ui.label("Subscriptions");
                                if elem.clicked() {
                                    debug!("Clicked on subscriptions");
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
                                        ui.checkbox(checked, sub.to_string());
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
