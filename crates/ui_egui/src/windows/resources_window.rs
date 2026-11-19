use crate::app::MyApp;
use crate::icons::resource_icon;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_iconography_azure::icon_for_resource_type;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::vec2;
use egui_extras::Column;
use egui_extras::TableBuilder;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

/// Browse the current account's resource inventory using local search and scope filters.
pub fn resources_ui(app: &mut MyApp, ui: &mut Ui) {
    if app.show_azure_authentication_required(ui) {
        return;
    }
    if matches!(app.resources, Loadable::NotLoaded) {
        load_resources(app, false);
    }

    let mut refresh = false;
    let mut sign_in = false;
    ui.horizontal(|ui| {
        refresh = ui
            .add_enabled(
                !matches!(app.resources, Loadable::Loading { .. }),
                Button::new("Refresh"),
            )
            .on_hover_text("Fetch a fresh inventory from Azure")
            .clicked();

        let selected_count = app
            .subscriptions
            .as_loaded()
            .map(|subscriptions| {
                subscriptions
                    .iter()
                    .filter(|subscription| {
                        app.checkboxes
                            .get(&Id::new(subscription.id))
                            .copied()
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0);
        let subscription_label = match selected_count {
            0 => "All subscriptions".to_owned(),
            1 => "1 subscription selected".to_owned(),
            count => format!("{count} subscriptions selected"),
        };
        ui.menu_button(subscription_label, |ui| {
            if matches!(app.subscriptions, Loadable::NotLoaded) {
                load_subscriptions(app);
            }
            match &app.subscriptions {
                Loadable::Loaded {
                    value: subscriptions,
                    ..
                } => {
                    let subscriptions = Rc::clone(subscriptions);
                    // These are the same selections shown by Starting Points and Selected Items.
                    if ui.button("Show all subscriptions").clicked() {
                        for subscription in subscriptions.iter() {
                            *app.checkbox_for(subscription.id) = false;
                        }
                    }
                    ui.separator();
                    ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                        for subscription in subscriptions.iter() {
                            ui.checkbox(
                                app.checkbox_for(subscription.id),
                                subscription.name.as_str(),
                            );
                        }
                    });
                    ui.small("With no subscriptions selected, all subscriptions are shown.");
                }
                Loadable::Failed { error, .. } => {
                    ui.label(format!("Could not load subscriptions: {error}"));
                    if ui.button("Retry loading subscriptions").clicked() {
                        load_subscriptions(app);
                    }
                }
                Loadable::NotLoaded | Loadable::Loading { .. } => {
                    ui.spinner();
                    ui.label("Loading subscriptions…");
                }
            }
        });
        if !app.resource_search.is_empty() && ui.button("Clear search").clicked() {
            app.resource_search.clear();
        }
    });
    ui.add_space(8.0);

    match &app.resources {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading resources…");
            });
        }
        Loadable::Failed { error, .. } => {
            ui.colored_label(
                Color32::from_rgb(164, 38, 44),
                "Resources could not be loaded.",
            );
            ui.horizontal(|ui| {
                refresh |= ui.button("Retry").clicked();
                sign_in = ui.button("Sign in again").clicked();
            });
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
        }
        Loadable::Loaded {
            value: resources, ..
        } => {
            // Resource Graph already supplies subscription containers alongside
            // resources, so their display names need no extra network request.
            let subscription_names: HashMap<_, _> = resources
                .iter()
                .filter(|resource| {
                    resource.kind == ResourceType::MICROSOFT_DOT_RESOURCES_SLASH_SUBSCRIPTIONS
                })
                .filter_map(|resource| {
                    resource_scope_columns(&resource.id.expanded_form())
                        .0
                        .map(|id| (id, resource.display_name().unwrap_or(&resource.name)))
                })
                .collect();
            let selected_subscriptions: HashSet<SubscriptionId> = app
                .subscriptions
                .as_loaded()
                .into_iter()
                .flat_map(|subscriptions| subscriptions.iter())
                .filter(|subscription| {
                    app.checkboxes
                        .get(&Id::new(subscription.id))
                        .copied()
                        .unwrap_or(false)
                })
                .map(|subscription| subscription.id)
                .collect();
            let search = app.resource_search.trim().to_lowercase();
            let resource_kind_filter = app.page.resource_type();
            let visible_resources: Vec<_> = resources
                .iter()
                .filter(|resource| {
                    if resource_kind_filter
                        .as_ref()
                        .is_some_and(|kind| kind != &resource.kind)
                    {
                        return false;
                    }
                    if search.is_empty() && selected_subscriptions.is_empty() {
                        return true;
                    }
                    let id = resource.id.expanded_form();
                    if !selected_subscriptions.is_empty()
                        && !resource_scope_columns(&id).0.is_some_and(|subscription| {
                            selected_subscriptions.contains(&subscription)
                        })
                    {
                        return false;
                    }
                    search.is_empty()
                        || resource.name.to_lowercase().contains(&search)
                        || resource.kind.as_ref().to_lowercase().contains(&search)
                        || id.to_lowercase().contains(&search)
                        || resource
                            .display_name()
                            .is_some_and(|name| name.to_lowercase().contains(&search))
                })
                .collect();

            ui.weak(format!(
                "Showing {} of {} resources",
                visible_resources.len(),
                resources.len()
            ));
            ui.add_space(8.0);
            if visible_resources.is_empty() {
                if resources.is_empty() {
                    ui.label("This account has no visible resources in the selected tenant.");
                } else {
                    ui.label("No resources match the selected service, search, and subscription filters.");
                }
            } else {
                let available_height = ui.available_height();
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.set_min_width(840.0);
                    TableBuilder::new(ui)
                        .striped(false)
                        .resizable(true)
                        .column(Column::remainder().at_least(180.0).clip(true))
                        .column(Column::initial(210.0).at_least(120.0).clip(true))
                        .column(Column::initial(200.0).at_least(100.0).clip(true))
                        .column(Column::initial(220.0).at_least(120.0).clip(true))
                        .min_scrolled_height(0.0)
                        .max_scroll_height((available_height - 16.0).max(0.0))
                        .header(30.0, |mut header| {
                            for title in ["Name", "Type", "Resource group", "Subscription"] {
                                header.col(|ui| {
                                    ui.strong(title);
                                });
                            }
                        })
                        .body(|body| {
                            body.rows(32.0, visible_resources.len(), |mut row| {
                                let resource = visible_resources[row.index()];
                                let id = resource.id.expanded_form();
                                let (subscription_id, resource_group) = resource_scope_columns(&id);
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            Image::new(resource_icon(&resource.kind))
                                                .fit_to_exact_size(vec2(18.0, 18.0)),
                                        );
                                        ui.add(
                                            Label::new(
                                                RichText::new(
                                                    resource
                                                        .display_name()
                                                        .unwrap_or(&resource.name),
                                                )
                                                .color(Color32::from_rgb(0, 120, 212)),
                                            )
                                            .truncate(),
                                        )
                                        .on_hover_text(&id)
                                        .context_menu(
                                            |ui| {
                                                if ui.button("Copy resource ID").clicked() {
                                                    ui.ctx().copy_text(id.clone());
                                                    ui.close();
                                                }
                                                if ui.button("Copy resource name").clicked() {
                                                    ui.ctx().copy_text(resource.name.clone());
                                                    ui.close();
                                                }
                                            },
                                        );
                                    });
                                });
                                row.col(|ui| {
                                    let type_label: Cow<'_, str> =
                                        match icon_for_resource_type(&resource.kind) {
                                            Some(icon) => icon.name(),
                                            None => Cow::Borrowed(resource.kind.as_ref()),
                                        };
                                    ui.add(Label::new(type_label.as_ref()).truncate())
                                        .on_hover_text(resource.kind.as_ref());
                                });
                                row.col(|ui| {
                                    ui.add(Label::new(resource_group.unwrap_or("—")).truncate());
                                });
                                row.col(|ui| {
                                    let subscription_name = app
                                        .subscriptions
                                        .as_loaded()
                                        .into_iter()
                                        .flat_map(|subscriptions| subscriptions.iter())
                                        .find(|subscription| {
                                            Some(subscription.id) == subscription_id
                                        })
                                        .map(|subscription| subscription.name.as_str())
                                        .or_else(|| {
                                            subscription_id
                                                .and_then(|id| subscription_names.get(&id).copied())
                                        });
                                    let label = subscription_name
                                        .map(str::to_owned)
                                        .or_else(|| subscription_id.map(|id| id.to_string()))
                                        .unwrap_or_else(|| "—".to_owned());
                                    ui.add(Label::new(label).truncate());
                                });
                            });
                        });
                });
            }
        }
    }
    if sign_in {
        app.open_selected_account_sign_in();
    }
    if refresh {
        load_resources(app, true);
    }
}

fn load_resources(app: &mut MyApp, refresh: bool) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let executor = app.executor.clone();
    let work_cache = app.work_cache.clone();
    let cancellation = executor.cancellation().clone();
    let generation = app.account_generation;
    let started_at = Instant::now();
    let work = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Fetch resources")
        .setter(move |app, resources| {
            if app.account_generation == generation {
                app.resources = resources;
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            Ok(Arc::new(
                AzureResourceListRequest { authentication }
                    .execute(&executor)
                    .with_invalidation(refresh)
                    .await?,
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
    if let Err(error) = work {
        app.resources = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}

/// Read table labels from the resource's scope, retaining absent scope as unknown.
/// Management-group and tenant resources do not acquire invented subscription identities.
fn resource_scope_columns(id: &str) -> (Option<SubscriptionId>, Option<&str>) {
    let mut parts = id.trim_start_matches('/').split('/');
    if !parts
        .next()
        .is_some_and(|part| part.eq_ignore_ascii_case("subscriptions"))
    {
        return (None, None);
    }
    let subscription_id = parts.next().and_then(|part| part.parse().ok());
    let resource_group = match (parts.next(), parts.next()) {
        (Some(segment), Some(name)) if segment.eq_ignore_ascii_case("resourceGroups") => Some(name),
        _ => None,
    };
    (subscription_id, resource_group)
}
