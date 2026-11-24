use crate::app::MyApp;
use crate::icons::STORAGE_EXPLORER_ICON;
use crate::inventory_table::show_inventory_result_count;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_search::portal_search;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use crate::storage_explorer::load_storage_explorer;
use crate::storage_explorer::load_storage_explorer_container;
use crate::storage_explorer_container_row::StorageExplorerContainerRow;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainer;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Grid;
use eframe::egui::Id;
use eframe::egui::ImageSource;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use std::collections::HashMap;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;
use teamy_tracing::gui::escape_display;

const COLUMNS: [ReflectedTableColumn; 4] = [
    ReflectedTableColumn::new("Name", "name")
        .with_link("id")
        .with_icon(container_icon),
    ReflectedTableColumn::new("Last modified", "last_modified").with_width(170.0),
    ReflectedTableColumn::new("Anonymous access", "anonymous_access").with_width(140.0),
    ReflectedTableColumn::new("Lease state", "lease_state").with_width(110.0),
];

pub(crate) fn storage_explorer_ui(app: &mut MyApp, ui: &mut Ui, id: &AzureStorageAccountId) {
    let view = ui.make_persistent_id(("storage-explorer", app.azure.account_generation, id));
    if !app.azure.storage_explorer.inventories.contains_key(id) {
        load_storage_explorer(app, id, false);
    }
    let inventory = match app.azure.storage_explorer.inventories.get(id) {
        Some(Loadable::Loaded { value, .. }) => value,
        Some(Loadable::Failed { error, .. }) => {
            if show_failure(ui, "Blob containers could not be loaded.", error) {
                load_storage_explorer(app, id, true);
            }
            return;
        }
        _ => {
            ui.spinner();
            return;
        }
    };

    ui.strong("Blob containers");
    if let Some(open) = show_inventory(ui, view, inventory) {
        open.apply(app);
    }
}

fn show_inventory(
    ui: &mut Ui,
    view: Id,
    rows: &[StorageExplorerContainerRow],
) -> Option<PortalTabOpen> {
    let prefix_id = view.with("prefix");
    let mut prefix = ui.data(|data| data.get_temp::<String>(prefix_id).unwrap_or_default());
    if portal_search(ui, &mut prefix, "Find containers by prefix", 320.0).changed() {
        ui.data_mut(|data| data.insert_temp(prefix_id, prefix.clone()));
    }
    let prefix = prefix.trim().to_ascii_lowercase();
    let filtered: Vec<_> = rows
        .iter()
        .filter(|row| row.id.container_name.as_str().starts_with(&prefix))
        .collect();
    show_inventory_result_count(ui, view, filtered.len(), rows.len());
    if rows.is_empty() {
        ui.label("No active blob containers.");
    } else if filtered.is_empty() {
        ui.label("No containers match this prefix.");
    }
    ReflectedTable::new(view, &COLUMNS)
        .without_selection()
        .show(ui, filtered, |row| view.with(&row.id), &mut HashMap::new())
}

pub(crate) fn storage_container_ui(
    app: &mut MyApp,
    ui: &mut Ui,
    id: &AzureStorageAccountBlobContainerId,
) {
    ui.heading(escape_display(id.container_name.as_str()));
    let explorer = ui.link("Back to blob containers");
    if PortalTabOpen::from_response(&explorer, || PortalTabTarget::ScopeView {
        scope: PortalResourceScope::StorageAccount(id.storage_account_id.clone()),
        view: PortalScopeView::StorageExplorer,
    })
    .is_some_and(|open| open.apply(app))
    {
        return;
    }
    if app.show_azure_authentication_required(ui) {
        return;
    }

    if !app.azure.storage_explorer.containers.contains_key(id) {
        load_storage_explorer_container(app, id, false);
    }
    match app.azure.storage_explorer.containers.get(id) {
        Some(Loadable::Loaded { value, .. }) => show_container_properties(ui, value),
        Some(Loadable::Failed { error, .. }) => {
            if show_failure(ui, "Container properties could not be loaded.", error) {
                load_storage_explorer_container(app, id, true);
            }
        }
        _ => {
            ui.spinner();
        }
    }
}

fn show_container_properties(ui: &mut Ui, container: &AzureStorageAccountBlobContainer) {
    let summary = StorageExplorerContainerRow::new(container);
    ScrollArea::vertical()
        .id_salt(("blob-container-properties", &container.id))
        .show(ui, |ui| {
            if ui.link("Copy resource ID").clicked() {
                ui.ctx().copy_text(container.id.to_string());
            }
            Grid::new(("blob-container-summary", &container.id))
                .num_columns(2)
                .show(ui, |ui| {
                    for (label, value) in [
                        ("Last modified", &summary.last_modified),
                        ("Anonymous access", &summary.anonymous_access),
                        ("Lease state", &summary.lease_state),
                    ] {
                        ui.label(label);
                        ui.label(value);
                        ui.end_row();
                    }
                });
            if let Some(metadata) = &container.properties.metadata
                && !metadata.is_empty()
            {
                ui.strong("Metadata");
                let mut metadata: Vec<_> = metadata.iter().collect();
                metadata.sort_unstable_by_key(|(key, _)| *key);
                Grid::new(("blob-container-metadata", &container.id))
                    .num_columns(2)
                    .show(ui, |ui| {
                        for (key, value) in metadata {
                            ui.label(escape_display(key));
                            ui.label(escape_display(value));
                            ui.end_row();
                        }
                    });
            }
            ui.collapsing("All properties", |ui| {
                ReflectedValue::new(&container.properties).show(ui);
            });
        });
}

fn show_failure(ui: &mut Ui, message: &str, error: &DiagnosticReport) -> bool {
    ui.label(message);
    ui.collapsing("Error details", |ui| {
        DiagnosticReportView::new(error).show(ui);
    });
    ui.button("Retry").clicked()
}

fn container_icon(_: &Ui, _: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
    Some(STORAGE_EXPLORER_ICON)
}
