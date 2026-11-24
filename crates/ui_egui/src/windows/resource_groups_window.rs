use crate::app::MyApp;
use crate::icons::RESOURCE_GROUP_ICON;
use crate::inventory_table::show_inventory_result_count;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::workers::load_resource_groups::load_resource_groups;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Id;
use eframe::egui::Ui;
use teamy_tracing::gui::DiagnosticReportView;

pub fn resource_groups_ui(app: &mut MyApp, ui: &mut Ui) {
    if app.show_azure_authentication_required(ui) {
        return;
    }
    if matches!(app.azure.resource_groups, Loadable::NotLoaded) {
        load_resource_groups(app, false);
    }

    match &app.azure.resource_groups {
        Loadable::Loaded { value, .. } => {
            let groups = value.clone();
            let columns = [
                ReflectedTableColumn::new("Name", "name")
                    .with_link("id")
                    .with_icon(|_, _| Some(RESOURCE_GROUP_ICON)),
                ReflectedTableColumn::new("Subscription", "subscription_name")
                    .with_width(220.0)
                    .with_link("id.subscription_id"),
                ReflectedTableColumn::new("Location", "location").with_width(132.0),
            ];
            let id = Id::new(("resource-groups", app.azure.account_generation));
            let table = ReflectedTable::new(id, &columns);
            if app
                .azure
                .resource_group_browser
                .filters
                .show(ui, groups.values().map(|group| group.as_ref()))
            {
                table.reset_selection_anchor(ui);
            }
            let rows: Vec<_> = app
                .azure
                .resource_group_browser
                .filters
                .filter(groups.values().map(|group| group.as_ref()))
                .collect();
            show_inventory_result_count(ui, id, rows.len(), groups.len());
            if rows.is_empty() {
                ui.label(if groups.is_empty() {
                    "This account has no visible resource groups."
                } else {
                    "No resource groups match these filters."
                });
            } else if let Some(target) = table.show(
                ui,
                rows,
                |group| Id::new(&group.id),
                &mut app.azure.checkboxes,
            ) {
                target.apply(app);
            }
        }
        Loadable::Failed { error, .. } => {
            ui.label("Resource groups could not be loaded.");
            let sign_in = ui.button("Sign in again").clicked();
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui)
            });
            if sign_in {
                app.open_selected_account_sign_in();
            }
        }
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.spinner();
        }
    }
}
