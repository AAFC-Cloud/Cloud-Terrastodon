use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use eframe::egui::Checkbox;
use eframe::egui::ScrollArea;
use eframe::egui::Widget;
use eframe::egui::Window;
use eframe::egui::collapsing_header::CollapsingState;
use tracing::debug;

use crate::app::MyApp;
use crate::app_message::AppMessage;
use crate::loadable::Loadable;
use crate::state_mutator::StateMutator;

impl MyApp {
    pub fn draw_app(&mut self, ctx: &eframe::egui::Context) {
        Window::new("Starting Points").show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let mut expando = CollapsingState::load_with_default_open(
                        ctx,
                        "subscriptions".into(),
                        self.toggle_subscriptions_expando,
                    );
                    if self.toggle_subscriptions_expando {
                        expando.toggle(&ui);
                        self.toggle_subscriptions_expando = false;
                    }
                    let is_open = expando.is_open();
                    if is_open && matches!(self.subscriptions, Loadable::NotLoaded) {
                        let tx = self.tx.clone();
                        #[derive(Debug)]
                        struct UpdateSubscriptions(Vec<Subscription>);
                        impl StateMutator for UpdateSubscriptions {
                            fn mutate_state(self: Box<Self>, state: &mut MyApp) {
                                state.subscriptions = Loadable::Loaded(
                                    self.0.into_iter().map(|x| (false, x)).collect(),
                                );
                            }
                        }
                        self.try_thing(async move {
                            let subscriptions = fetch_all_subscriptions().await?;
                            tx.send(AppMessage::update_state(UpdateSubscriptions(subscriptions)))?;
                            Ok(())
                        });
                        self.subscriptions = Loadable::Loading;
                    }

                    expando
                        .clone()
                        .show_header(ui, |ui| match &mut self.subscriptions {
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
                                    self.toggle_subscriptions_expando = true;
                                };
                            }
                        })
                        .body(|ui| {
                            ui.vertical(|ui| match &mut self.subscriptions {
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
                            });
                        });
                })
            });
        });
    }
}
