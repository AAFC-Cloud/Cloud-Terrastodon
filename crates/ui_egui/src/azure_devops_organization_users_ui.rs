use crate::app::MyApp;
use crate::azure_devops_organization_user_row::AzureDevOpsOrganizationUserRow;
use crate::azure_devops_user_license_draft::AzureDevOpsUserLicenseDraft;
use crate::entra_directory_ui::loaded;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Ui;
use std::collections::HashMap;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn show_azure_devops_organization_users(
    app: &mut MyApp,
    ui: &mut Ui,
    organization: &AzureDevOpsOrganizationUrl,
) {
    if organization.organization_name().is_err() {
        ui.label("User licenses are available for Azure DevOps Services organizations.");
        return;
    }
    app.show_azure_devops_credentials_for(ui, organization);
    let Some(authentication) = app.ready_azure_devops_authentication_for(organization) else {
        app.show_azure_devops_authentication_required(ui, organization);
        return;
    };
    if matches!(
        app.azure
            .azure_devops_users
            .users(organization, authentication),
        Loadable::NotLoaded
    ) {
        app.load_azure_devops_users(organization, false);
    }

    let id = ui.make_persistent_id((
        "devops-organization-users",
        app.tabs.active_id(),
        organization,
        authentication,
        app.azure.account_generation,
    ));
    let pending = app.azure_devops_license_update_in_flight(organization);
    let demo_mode = app.egui_config.demo_mode;
    ui.push_id(id, |ui| {
        let draft_id = id.with("license-draft");
        if pending {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Updating access level…");
            });
        } else {
            match app
                .azure
                .azure_devops_users
                .update(organization, authentication)
            {
                Some(Loadable::Loaded { value, .. }) => {
                    ui.label(value);
                }
                Some(Loadable::Failed { error, .. }) => {
                    ui.label("Could not complete the access level change.");
                    ui.collapsing("Change details", |ui| {
                        DiagnosticReportView::new(error).show(ui)
                    });
                }
                _ => {}
            }
        }
        let Some(users) = loaded(
            ui,
            app.azure
                .azure_devops_users
                .users(organization, authentication),
            "Could not load the organization's users.",
        ) else {
            return;
        };

        let action = ReflectedTableColumn::new("Actions", "action").with_width(138.0);
        let columns = [
            ReflectedTableColumn::new("Name", "name"),
            ReflectedTableColumn::new("Email / sign-in", "sign_in").with_width(215.0),
            ReflectedTableColumn::new("Access level", "license")
                .with_width(150.0)
                .groupable(),
            ReflectedTableColumn::new("Assignment", "assignment_source")
                .with_width(110.0)
                .groupable(),
            ReflectedTableColumn::new("Status", "status")
                .with_width(104.0)
                .groupable(),
            if pending {
                action
            } else {
                action.with_row_link()
            },
        ];
        let rows = users
            .iter()
            .map(|entitlement| AzureDevOpsOrganizationUserRow::new(entitlement, demo_mode))
            .collect::<Vec<_>>();
        let mut selected = None;
        ReflectedTable::new("organization-users", &columns)
            .without_selection()
            .with_search_hint("Search users or access levels")
            .with_result_count()
            .show_explorable_with_activation(
                ui,
                &rows,
                |row| id.with(row.id),
                &mut HashMap::new(),
                |row| selected = Some(row.id),
            );
        let mut draft = selected
            .and_then(|id| users.iter().find(|user| user.id == id))
            .map(|entitlement| AzureDevOpsUserLicenseDraft::new(entitlement))
            .or_else(|| ui.data(|data| data.get_temp::<AzureDevOpsUserLicenseDraft>(draft_id)));
        let Some(draft) = draft.as_mut() else {
            return;
        };
        let entitlement = users
            .iter()
            .find(|user| user.id == draft.user_id)
            .map(AsRef::as_ref);
        let (new_license, close) =
            draft.show(ui, draft_id, organization, entitlement, demo_mode, pending);
        if let Some(new_license) = new_license {
            app.update_azure_devops_user_license(
                organization,
                authentication,
                draft.user_id,
                draft.expected_license.clone(),
                new_license,
            );
        } else if !close {
            ui.data_mut(|data| data.insert_temp(draft_id, draft.clone()));
            return;
        }
        ui.data_mut(|data| data.remove::<AzureDevOpsUserLicenseDraft>(draft_id));
    });
}
