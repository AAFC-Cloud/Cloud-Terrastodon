use crate::app::MyApp;
use crate::icons::DEVOPS_PROJECT_ICON;
use crate::inventory_table::show_inventory_result_count;
use crate::portal_tab_open::PortalTabOpen;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::windows::azure_devops_project_row::AzureDevOpsProjectRow;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::collections::HashMap;
use std::sync::Arc;
use teamy_tracing::gui::DiagnosticReportView;

/// Browse projects in the selected organization with its chosen credential.
pub(crate) fn azure_devops_projects_ui(
    app: &mut MyApp,
    ui: &mut Ui,
    organization: &AzureDevOpsOrganizationUrl,
) {
    app.show_azure_devops_credentials_for(ui, organization);
    let Some(authentication) = app.ready_azure_devops_authentication_for(organization) else {
        app.show_azure_devops_authentication_required(ui, organization);
        return;
    };

    if matches!(
        app.azure
            .azure_devops_projects
            .get(organization, authentication),
        Loadable::NotLoaded
    ) {
        app.load_azure_devops_projects_for(organization, false);
    }

    match app
        .azure
        .azure_devops_projects
        .get(organization, authentication)
    {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading projects…");
            });
        }
        Loadable::Failed { error, .. } => {
            ui.label("Could not load projects.");
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
        }
        Loadable::Loaded {
            value: projects, ..
        } => {
            let project_table = ui.make_persistent_id((
                "azure_devops_projects",
                app.azure.account_generation,
                organization,
                authentication,
            ));
            if let Some(open) = show_projects_table(
                ui,
                organization,
                authentication,
                projects,
                project_table,
                &mut app.azure.checkboxes,
            ) {
                open.apply(app);
            }
        }
    }
}

fn show_projects_table(
    ui: &mut Ui,
    organization: &AzureDevOpsOrganizationUrl,
    authentication: AzureDevOpsAuthSelection,
    projects: &[Arc<AzureDevOpsProject>],
    id: Id,
    checked_rows: &mut HashMap<Id, bool>,
) -> Option<PortalTabOpen> {
    if projects.is_empty() {
        show_inventory_result_count(ui, id, 0, 0);
        ui.label("No projects are visible in this organization.");
        return None;
    }
    let columns = [
        ReflectedTableColumn::new("Name", "name")
            .with_icon(|_, _| Some(DEVOPS_PROJECT_ICON))
            .with_identifier("id")
            .with_link("target"),
        ReflectedTableColumn::new("Visibility", "visibility")
            .with_width(106.0)
            .groupable(),
        ReflectedTableColumn::new("Last updated", "last_updated").with_width(206.0),
    ];
    let rows: Vec<_> = projects
        .iter()
        .map(|project| AzureDevOpsProjectRow::new(organization, project))
        .collect();
    ReflectedTable::new(id, &columns)
        .with_search_hint("Search projects")
        .with_result_count()
        .show_explorable(
            ui,
            &rows,
            |row| Id::new((organization, authentication, row.id)),
            checked_rows,
        )
}
