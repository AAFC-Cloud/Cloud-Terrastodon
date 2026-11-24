use crate::app::MyApp;
use crate::icons::DEVOPS_ICON;
use crate::inventory_table::show_inventory_result_count;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::windows::azure_devops_organization_row::AzureDevOpsOrganizationRow;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::collections::HashMap;

pub(crate) fn azure_devops_organizations_ui(app: &mut MyApp, ui: &mut Ui) {
    let tracked_organizations_loaded = app.show_tracked_azure_devops_organizations_status(ui);
    if let Some(organization) = app.azure_devops_organization_picker.organization_input(ui) {
        app.navigate_tab(PortalAzureDevOpsOrganizationView::Projects.target(&organization));
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
    ];
    let account_tenant = app
        .azure
        .authentication
        .map(|selection| selection.tenant_id);
    let rows: Vec<_> = organizations
        .into_iter()
        .map(|(organization, tenant_id)| {
            AzureDevOpsOrganizationRow::new(organization, tenant_id, account_tenant)
        })
        .collect();
    if let Some(open) = ReflectedTable::new(id, &columns)
        .without_selection()
        .with_deemphasized_rows(|row| {
            row.get::<AzureDevOpsOrganizationRow>()
                .is_some_and(|row| row.different_tenant)
        })
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
