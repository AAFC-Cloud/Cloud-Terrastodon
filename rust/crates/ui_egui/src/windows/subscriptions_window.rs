use crate::app::MyApp;
use crate::loadable::Loadable;
use crate::workers::load_resource_groups::load_resource_groups;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure::prelude::SubscriptionId;
use eframe::egui;
use eframe::egui::Checkbox;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Widget;
use eframe::egui::Window;
use eframe::egui::collapsing_header::CollapsingState;
use std::hash::Hash;
use tracing::debug;

pub fn draw_subscriptions_window(app: &mut MyApp, ctx: &Context) {
    Window::new("Starting Points").show(ctx, |ui| {
        ScrollArea::both().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                draw_subscription_list_expando(app, ctx, ui);
            })
        });
    });
}

fn draw_subscription_list_expando(app: &mut MyApp, ctx: &Context, ui: &mut Ui) {
    let mut expando = CollapsingState::load_with_default_open(ctx, "subscriptions".into(), false);
    let toggle_key = expando.id();
    if app.toggle_intents.remove(&toggle_key) {
        expando.toggle(&ui);
    }
    let is_open = expando.is_open();
    if is_open && matches!(app.subscriptions, Loadable::NotLoaded) {
        load_subscriptions(app);
    }
    expando
        .clone()
        .show_header(ui, |ui| {
            draw_subscription_list_expando_header(app, ui, toggle_key)
        })
        .body(|ui| draw_subscription_list_expando_body(app, ctx, ui));
}

fn draw_subscription_list_expando_header(app: &mut MyApp, ui: &mut Ui, toggle_key: Id) {
    match &app.subscriptions {
        Loadable::Loaded(subs) => {
            let subs = subs.clone();
            if ui
                .image(egui::include_image!(
                    "../../assets/10002-icon-service-Subscriptions-4x.png"
                ))
                .clicked()
            {
                debug!("Clicked on subscription icon");
                app.toggle_intents.insert(toggle_key);
            }

            let mut all = subs.iter().all(|sub| *app.checkbox_for(&sub.id));
            let any = subs.iter().any(|sub| *app.checkbox_for(&sub.id));
            let indeterminate = any && !all;
            let elem = Checkbox::new(&mut all, "Subscriptions")
                .indeterminate(indeterminate)
                .ui(ui);
            if elem.changed() {
                for sub in subs.iter() {
                    *app.checkbox_for(&sub.id) = all
                }
            }
        }
        _ => {
            if ui
                .image(egui::include_image!(
                    "../../assets/10002-icon-service-Subscriptions-4x.png"
                ))
                .clicked()
            {
                debug!("Clicked on subscription header icon");
                app.toggle_intents.insert(toggle_key);
            }
            let elem = ui.label("Subscriptions");
            if elem.clicked() {
                debug!("Clicked on subscriptions header text");
                app.toggle_intents.insert(toggle_key);
            };
        }
    }
}

fn draw_subscription_list_expando_body(app: &mut MyApp, ctx: &Context, ui: &mut Ui) {
    ui.vertical(|ui| match &app.subscriptions {
        Loadable::NotLoaded => {
            ui.label("Not loaded");
        }
        Loadable::Loading => {
            ui.label("Loading...");
        }
        Loadable::Loaded(subs) => {
            let subs = subs.clone();
            for subscription in subs.iter() {
                draw_subscription_list_expando_body_entry(app, ctx, ui, subscription);
            }
        }
        Loadable::Failed(err) => {
            ui.label(&format!("Error: {}", err));
        }
    });
}

fn draw_subscription_list_expando_body_entry(
    app: &mut MyApp,
    ctx: &Context,
    ui: &mut Ui,
    subscription: &Subscription,
) {
    draw_subscription_expando(app, ctx, ui, subscription);
}

fn draw_subscription_expando(
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

// fn draw_resource_groups_expando(
//     app: &mut MyApp,
//     ctx: &Context,
//     ui: &mut Ui,
//     subscription: &Subscription,
// ) {
//     let mut expando =
//         CollapsingState::load_with_default_open(ctx, Id::new(&subscription.id), false);
//     let toggle_key = expando.id();
//     if app.toggle_intents.remove(&toggle_key) {
//         expando.toggle(&ui);
//     }
//     let is_open = expando.is_open();
//     if is_open && matches!(app.resource_groups, Loadable::NotLoaded) {
//         load_resource_groups(app);
//     }
//     expando
//         .clone()
//         .show_header(ui, |ui| {
//             draw_resource_groups_expando_header(app, ctx, ui, &subscription.id, toggle_key)
//         })
//         .body(|ui| draw_resource_groups_expando_body(app, ctx, ui, &subscription.id, toggle_key));
// }

// fn draw_resource_groups_expando_header(
//     app: &mut MyApp,
//     ctx: &Context,
//     ui: &mut Ui,
//     subscription_id: &SubscriptionId,
//     toggle_key: Id,
// ) {
//     match &mut app.resource_groups {
//         Loadable::Loaded(resource_groups) => match resource_groups.get_mut(subscription_id) {
//             None => {
//                 ui.label("No resource groups for this subscription");
//             }
//             Some(resource_groups) => {
//                 if ui
//                     .image(egui::include_image!(
//                         "../../assets/10007-icon-service-Resource-Groups-4x.png"
//                     ))
//                     .clicked()
//                 {
//                     debug!("Clicked on resource group icon");
//                     app.toggle_intents.insert(toggle_key);
//                 }

//                 let mut all = resource_groups.iter().all(|(checked, _)| *checked);
//                 let any = resource_groups.iter().any(|(checked, _)| *checked);
//                 let indeterminate = any && !all;
//                 let elem = Checkbox::new(&mut all, "Resource Groups")
//                     .indeterminate(indeterminate)
//                     .ui(ui);
//                 if elem.changed() {
//                     for (checked, _) in resource_groups.iter_mut() {
//                         *checked = all;
//                     }
//                 }
//             }
//         },
//         _ => {
//             if ui
//                 .image(egui::include_image!(
//                     "../../assets/10007-icon-service-Resource-Groups-4x.png"
//                 ))
//                 .clicked()
//             {
//                 debug!("Clicked on resource groups header icon");
//                 app.toggle_intents.insert(toggle_key);
//             }
//             let elem = ui.label("Resource Groups");
//             if elem.clicked() {
//                 debug!("Clicked on resource groups header text");
//                 app.toggle_intents.insert(toggle_key);
//             };
//         }
//     }
// }
