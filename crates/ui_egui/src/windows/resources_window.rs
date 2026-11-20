use crate::app::MyApp;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::resource_inventory_grouping::ResourceInventoryGrouping;
use crate::resource_inventory_row::ResourceInventoryRow;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::Align;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::ComboBox;
use eframe::egui::Id;
use eframe::egui::Layout;
use eframe::egui::ScrollArea;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

/// Browse the current account's inventory using local search and scope filters.
pub fn resources_ui(app: &mut MyApp, ui: &mut Ui) {
    let kind = app.current_page().resource_type();
    resources_filtered_ui(app, ui, kind.as_ref());
}

pub(crate) fn resources_of_type_ui(app: &mut MyApp, ui: &mut Ui, kind: &ResourceType) {
    resources_filtered_ui(app, ui, Some(kind));
}

fn resources_filtered_ui(app: &mut MyApp, ui: &mut Ui, kind: Option<&ResourceType>) {
    if app.show_azure_authentication_required(ui) {
        return;
    }
    if matches!(app.resources, Loadable::NotLoaded) {
        load_resources(app, false);
    }

    let mut refresh = false;
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
    let search = app.resource_search.trim().to_lowercase();
    if let Some(resources) = inventory(app, ui) {
        let selected_subscriptions: HashSet<_> = app
            .subscriptions
            .as_loaded()
            .into_iter()
            .flat_map(|subscriptions| subscriptions.iter())
            .filter(|subscription| app.checkboxes.get(&Id::new(subscription.id)) == Some(&true))
            .map(|subscription| subscription.id)
            .collect();
        show_inventory(
            app,
            ui,
            &resources,
            kind,
            None,
            &search,
            &selected_subscriptions,
        );
    }
    if refresh {
        load_resources(app, true);
    }
}

pub(crate) fn scoped_resources_ui(app: &mut MyApp, ui: &mut Ui, scope: &ScopeImpl) {
    if matches!(app.resources, Loadable::NotLoaded) {
        load_resources(app, false);
    }
    let filter_id = ui.id().with(("scoped-resource-filter", scope));
    let mut search = ui
        .data(|data| data.get_temp::<String>(filter_id))
        .unwrap_or_default();
    ui.heading("Resources");
    if ui
        .add(TextEdit::singleline(&mut search).hint_text("Filter resources"))
        .changed()
    {
        ui.data_mut(|data| data.insert_temp(filter_id, search.clone()));
    }
    if let Some(resources) = inventory(app, ui) {
        show_inventory(
            app,
            ui,
            &resources,
            None,
            Some(scope),
            &search.trim().to_lowercase(),
            &HashSet::new(),
        );
    }
}

fn inventory(app: &mut MyApp, ui: &mut Ui) -> Option<Arc<Vec<Arc<Resource>>>> {
    match &app.resources {
        Loadable::Loaded { value, .. } => Some(Arc::clone(value)),
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading resources…");
            });
            None
        }
        Loadable::Failed { error, .. } => {
            ui.colored_label(
                Color32::from_rgb(164, 38, 44),
                "Resources could not be loaded.",
            );
            let (retry, sign_in) = ui
                .horizontal(|ui| {
                    (
                        ui.button("Retry").clicked(),
                        ui.button("Sign in again").clicked(),
                    )
                })
                .inner;
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
            if retry {
                load_resources(app, true);
            } else if sign_in {
                app.open_selected_account_sign_in();
            }
            None
        }
    }
}

fn show_inventory(
    app: &mut MyApp,
    ui: &mut Ui,
    resources: &[Arc<Resource>],
    kind: Option<&ResourceType>,
    scope: Option<&ScopeImpl>,
    search: &str,
    selected_subscriptions: &HashSet<SubscriptionId>,
) {
    let mut subscription_names: HashMap<_, _> = resources
        .iter()
        .filter_map(|resource| match &resource.id {
            ScopeImpl::Subscription(id) => {
                Some((*id, resource.display_name().unwrap_or(&resource.name)))
            }
            _ => None,
        })
        .collect();
    if let Some(subscriptions) = app.subscriptions.as_loaded() {
        subscription_names.extend(
            subscriptions
                .iter()
                .map(|subscription| (subscription.id, subscription.name.as_str())),
        );
    }
    let rows: Vec<_> = resources
        .iter()
        .filter_map(|resource| {
            if kind.is_some_and(|kind| kind != &resource.kind) {
                return None;
            }
            let id = resource.id.expanded_form();
            let (subscription, group) = ResourceInventoryRow::ancestors(&id);
            if scope.is_some_and(|scope| {
                !ResourceInventoryRow::belongs_to(&resource.id, subscription, group.as_ref(), scope)
            }) || (!selected_subscriptions.is_empty()
                && !subscription.is_some_and(|id| selected_subscriptions.contains(&id)))
            {
                return None;
            }
            let row = ResourceInventoryRow::new(
                resource,
                subscription,
                subscription.and_then(|id| subscription_names.get(&id).copied()),
                group,
            );
            (search.is_empty()
                || row.name.to_lowercase().contains(search)
                || row.type_label.to_lowercase().contains(search)
                || resource.kind.as_ref().to_lowercase().contains(search)
                || row
                    .location
                    .as_ref()
                    .is_some_and(|location| location.to_string().to_lowercase().contains(search))
                || id.to_lowercase().contains(search))
            .then_some(row)
        })
        .collect();

    let inventory_id = Id::new((
        "resource-inventory",
        app.current_page().title(),
        scope,
        kind,
    ));
    let grouping_id = ui.make_persistent_id(inventory_id.with("grouping"));
    let previous_grouping = ui.data(|data| data.get_temp::<ResourceInventoryGrouping>(grouping_id));
    let mut grouping = previous_grouping;
    ui.horizontal(|ui| {
        ui.weak(format!("{} resources", rows.len()));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ComboBox::from_id_salt(grouping_id)
                .selected_text(grouping.map_or("None", ResourceInventoryGrouping::label))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut grouping, None, "None");
                    for option in ResourceInventoryGrouping::ALL {
                        ui.selectable_value(&mut grouping, Some(option), option.label());
                    }
                });
            ui.label("Group by");
        });
    });
    if grouping != previous_grouping {
        ui.data_mut(|data| match grouping {
            Some(grouping) => {
                data.insert_temp(grouping_id, grouping);
            }
            None => data.remove::<ResourceInventoryGrouping>(grouping_id),
        });
    }
    if rows.is_empty() {
        ui.label("No resources match this view.");
    }
    let mut columns = vec![
        ReflectedTableColumn::new("Name", "name")
            .with_link("id")
            .with_icon(ResourceInventoryRow::icon),
        ReflectedTableColumn::new("Type", "type_label")
            .with_width(176.0)
            .with_link("kind"),
    ];
    if !matches!(scope, Some(ScopeImpl::ResourceGroup(_))) {
        columns.push(
            ReflectedTableColumn::new("Resource group", "group_name")
                .with_width(168.0)
                .with_link("group_id"),
        );
    }
    columns.push(ReflectedTableColumn::new("Location", "location").with_width(134.0));
    if scope.is_none() {
        columns.push(
            ReflectedTableColumn::new("Subscription", "subscription_name")
                .with_width(168.0)
                .with_link("subscription_id"),
        );
    }
    let table = ReflectedTable::new(inventory_id.with(app.account_generation), &columns);
    if grouping != previous_grouping {
        table.reset_selection_anchor(ui);
    }
    let row_id = |row: &ResourceInventoryRow| Id::new(("resource-row", &row.id));
    let navigation = if let Some(grouping) = grouping {
        table.show_grouped(ui, rows.iter(), row_id, &mut app.checkboxes, |row| {
            grouping.group(row)
        })
    } else {
        table.show(ui, rows.iter(), row_id, &mut app.checkboxes)
    };
    if let Some(navigation) = navigation {
        app.navigate_tab(navigation);
    }
}

pub(crate) fn load_resources(app: &mut MyApp, refresh: bool) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let executor = app.executor.clone();
    let work_cache = app.work_cache.clone();
    let cancellation = executor.cancellation().clone();
    let generation = app.account_generation;
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, _>::new()
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
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = enqueue_result {
        app.resources = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}
