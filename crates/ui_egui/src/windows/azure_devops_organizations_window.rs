use crate::app::MyApp;
use crate::icons::DEVOPS_ICON;
use crate::inventory_table::show_inventory_result_count;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::windows::azure_devops_organization_row::AzureDevOpsOrganizationRow;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::collections::HashMap;

pub(crate) fn azure_devops_organizations_ui(app: &mut MyApp, ui: &mut Ui) {
    let tracked_organizations_loaded = app.show_tracked_azure_devops_organizations_status(ui);
    ui.horizontal_wrapped(|ui| app.show_track_azure_devops_organization_button(ui));
    ui.add_space(8.0);
    if app
        .azure_devops_organization_picker
        .is_loading_tracked_organizations()
    {
        return;
    }
    let organizations = app.azure_devops_organization_picker.known_organizations(
        app.azure.azure_devops_organization.as_ref(),
        app.account_picker.catalogue.as_ref().ok(),
    );
    let id = "azure_devops_organizations";
    if organizations.is_empty() {
        if tracked_organizations_loaded {
            show_inventory_result_count(ui, id, 0, 0);
            ui.label("No known Azure DevOps organizations.");
        }
        return;
    }
    let columns = [
        ReflectedTableColumn::new("Organization", "name")
            .with_width(220.0)
            .with_icon(|_, _| Some(DEVOPS_ICON))
            .with_link("organization"),
        ReflectedTableColumn::new("URL", "url"),
        ReflectedTableColumn::new("Tenant ID", "tenant_id").with_width(292.0),
        ReflectedTableColumn::new("Tracked", "tracked").with_width(100.0),
    ];
    let rows: Vec<_> = organizations
        .into_iter()
        .map(|(organization, tenant_id)| {
            let tracked = app
                .azure_devops_organization_picker
                .tracked_organization(organization)
                .is_some();
            AzureDevOpsOrganizationRow::new(organization, tenant_id, tracked)
        })
        .collect();
    if let Some(open) = ReflectedTable::new(id, &columns)
        .without_selection()
        .with_search_hint("Search organizations")
        .with_result_count()
        .show_explorable(
            ui,
            &rows,
            |row| Id::new(&row.organization),
            &mut HashMap::new(),
        )
    {
        open.apply(app);
    }
}
