use crate::app::MyApp;
use crate::azure_tag_inventory::AzureTagInventory;
use crate::azure_tag_inventory_entry::AzureTagInventoryEntry;
use crate::azure_tag_inventory_row::AzureTagInventoryRow;
use crate::icons::TAG_ICON;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use crate::resource_inventory_row::ResourceInventoryRow;
use crate::windows::resources_window::load_resources;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::ComboBox;
use eframe::egui::Id;
use eframe::egui::ImageSource;
use eframe::egui::Ui;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn azure_tags_inventory_ui(app: &mut MyApp, ui: &mut Ui) {
    if app.show_azure_authentication_required(ui) {
        return;
    }
    if matches!(app.azure.resources, Loadable::NotLoaded) {
        load_resources(app, false);
    }
    let resources = match &app.azure.resources {
        Loadable::Loaded { value, .. } => Arc::clone(value),
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading tags…");
            });
            return;
        }
        Loadable::Failed { error, .. } => {
            ui.label("Tags could not be loaded.");
            let sign_in = ui.button("Sign in again").clicked();
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
            if sign_in {
                app.open_selected_account_sign_in();
            }
            return;
        }
    };

    let cache_id = Id::new("tags-inventory-index");
    let inventory = ui
        .data(|data| data.get_temp::<Arc<AzureTagInventory>>(cache_id))
        .filter(|index| index.matches_snapshot(&resources))
        .unwrap_or_else(|| {
            let index = Arc::new(AzureTagInventory::new(&resources));
            ui.data_mut(|data| data.insert_temp(cache_id, Arc::clone(&index)));
            index
        });

    ui.push_id(("tags-inventory", app.azure.account_generation), |ui| {
        let subscription_id = ui.make_persistent_id("subscription");
        let selected_id = ui.make_persistent_id("selected-tag");
        let mut subscription = ui
            .data(|data| data.get_temp::<SubscriptionId>(subscription_id))
            .filter(|id| inventory.subscriptions.contains_key(id));
        let selected = ui.data(|data| data.get_temp::<(String, String)>(selected_id));
        let detail = selected
            .as_ref()
            .and_then(|selected| inventory.entries.get(selected));

        ui.horizontal_wrapped(|ui| {
            let previous_subscription = subscription;
            ComboBox::from_id_salt("subscription-picker")
                .selected_text(
                    subscription
                        .and_then(|id| inventory.subscriptions.get(&id).map(String::as_str))
                        .unwrap_or("All subscriptions"),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut subscription, None, "All subscriptions");
                    for (id, name) in inventory
                        .subscriptions
                        .iter()
                        .sorted_by_key(|(_, name)| name.as_str())
                    {
                        ui.selectable_value(&mut subscription, Some(*id), name);
                    }
                });
            if subscription != previous_subscription {
                ui.data_mut(|data| {
                    if let Some(subscription) = subscription {
                        data.insert_temp(subscription_id, subscription);
                    } else {
                        data.remove::<SubscriptionId>(subscription_id);
                    }
                });
            }
            if detail.is_none() {
                ui.hyperlink_to(
                    "Learn about Azure tags",
                    "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/tag-resources",
                );
            }
        });

        if let Some(((_, value), entry)) = selected.as_ref().zip(detail) {
            if ui.link("All tags").clicked() {
                ui.data_mut(|data| data.remove::<(String, String)>(selected_id));
                ui.ctx().request_repaint();
            }
            ui.heading(format!("{} = {}", entry.name, tag_value_label(value)));
            show_tagged_resources(app, ui, &resources, &inventory, entry, subscription);
        } else if let Some(tag) = show_tag_entries(ui, &inventory, subscription) {
            ui.data_mut(|data| data.insert_temp(selected_id, tag));
            ui.ctx().request_repaint();
        }
    });
}

fn show_tag_entries(
    ui: &mut Ui,
    inventory: &AzureTagInventory,
    subscription: Option<SubscriptionId>,
) -> Option<(String, String)> {
    let rows = inventory.entries.iter().filter_map(|(tag, entry)| {
        let resource_count = entry.resource_count(subscription);
        (resource_count != 0).then(|| AzureTagInventoryRow {
            key: tag.clone(),
            name: entry.name.clone(),
            value: tag_value_label(&tag.1).to_owned(),
            resource_count,
        })
    });
    let rows: Vec<_> = rows.collect();
    let columns = [
        ReflectedTableColumn::new("Name", "name")
            .with_width(220.0)
            .with_icon(tag_icon)
            .with_row_link(),
        ReflectedTableColumn::new("Value", "value").with_row_link(),
        ReflectedTableColumn::new("Resources", "resource_count").with_width(104.0),
    ];
    let mut selected = None;
    ReflectedTable::new("tags", &columns)
        .without_selection()
        .with_result_count()
        .with_search_hint("Search tag names or values")
        .show_explorable_with_activation(
            ui,
            &rows,
            |row| Id::new(&row.key),
            &mut HashMap::new(),
            |row| selected = Some(row.key.clone()),
        );
    selected
}

fn tag_icon(_: &Ui, _: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
    Some(TAG_ICON)
}

fn show_tagged_resources(
    app: &mut MyApp,
    ui: &mut Ui,
    resources: &[Arc<Resource>],
    inventory: &AzureTagInventory,
    entry: &AzureTagInventoryEntry,
    subscription: Option<SubscriptionId>,
) {
    let rows: Vec<_> = entry
        .resources(subscription)
        .filter_map(|(index, subscription)| {
            let resource = resources.get(index)?;
            let (_, group) = ResourceInventoryRow::ancestors(&resource.id.expanded_form());
            let subscription_name =
                subscription.and_then(|id| inventory.subscriptions.get(&id).map(String::as_str));
            Some(ResourceInventoryRow::new(
                resource,
                subscription,
                subscription_name,
                group,
            ))
        })
        .collect();
    let columns = [
        ReflectedTableColumn::new("Name", "name")
            .with_link("id")
            .with_icon(ResourceInventoryRow::icon),
        ReflectedTableColumn::new("Type", "type_label")
            .with_width(140.0)
            .with_link("kind"),
        ReflectedTableColumn::new("Subscription", "subscription_name")
            .with_width(160.0)
            .with_link("subscription_id"),
    ];
    let table = ReflectedTable::new("tag-resources", &columns)
        .with_result_count()
        .with_search_hint("Search tagged resources");
    if let Some(target) = table.show_explorable(
        ui,
        &rows,
        |row| Id::new(("resource-row", &row.id)),
        &mut app.azure.checkboxes,
    ) {
        target.apply(app);
    }
}

fn tag_value_label(value: &str) -> &str {
    if value.is_empty() { "(empty)" } else { value }
}
