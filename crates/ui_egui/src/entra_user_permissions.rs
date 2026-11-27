mod entra_user_azure_role_assignments;

pub(crate) use crate::entra_devops_permissions::show_entra_user_devops_permissions;

use self::entra_user_azure_role_assignments::EntraUserAzureRoleAssignments;
use crate::app::MyApp;
use crate::capture_gui_failure::capture_gui_failure;
use crate::entra_devops_permissions::EntraDevOpsPermissionOrganization;
use crate::entra_directory_ui::loaded;
use crate::entra_role_assignment_row::EntraRoleAssignmentRow;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_azure::EntraUserAzureRoleAssignmentListRequest;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::CollapsingHeader;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::collections::HashMap;
use teamy_tracing::gui::DiagnosticReportView;

/// Permission discovery results for users, scoped to the tab's selected Entra account.
#[derive(Default)]
pub(crate) struct EntraUserPermissions {
    azure_roles: HashMap<EntraUserId, Loadable<EntraUserAzureRoleAssignments>>,
    pub(crate) devops: HashMap<EntraUserId, Loadable<Vec<EntraDevOpsPermissionOrganization>>>,
}

impl EntraUserPermissions {
    pub(crate) fn is_loading(&self) -> bool {
        self.azure_roles.values().any(is_loading) || self.devops.values().any(is_loading)
    }

    pub(crate) fn can_refresh_azure_roles(&self, user_id: EntraUserId) -> bool {
        !self.azure_roles.get(&user_id).is_some_and(is_loading)
    }

    pub(crate) fn can_refresh_devops_permissions(&self, user_id: EntraUserId) -> bool {
        !self.devops.get(&user_id).is_some_and(is_loading)
    }
}

impl MyApp {
    pub(crate) fn load_entra_user_azure_roles(&mut self, user_id: EntraUserId, refresh: bool) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        match self.azure.entra_user_permissions.azure_roles.get(&user_id) {
            Some(Loadable::Loading { .. }) => return,
            None | Some(Loadable::NotLoaded) => {}
            _ if !refresh => return,
            _ => {}
        }

        let _ = self.queue_entra_work(
            "Load Azure role assignments for this user",
            move |azure, load| {
                azure
                    .entra_user_permissions
                    .azure_roles
                    .insert(user_id, load);
            },
            async move |executor| {
                let report = EntraUserAzureRoleAssignmentListRequest {
                    authentication,
                    user_id,
                }
                .execute(&executor)
                .with_invalidation(refresh)
                .await?;

                let rows = report
                    .assignments
                    .into_iter()
                    .map(|record| {
                        let mut row = EntraRoleAssignmentRow::from((
                            record.assignment.as_ref(),
                            record.role_definition.as_ref(),
                        ));
                        if record.assigned_group.is_some() {
                            row.assigned_via = "Group";
                            row.assigned_group = record.assigned_group;
                        }
                        row
                    })
                    .collect();
                let membership_error = report.membership_failure.map(|failure| {
                    capture_gui_failure(
                        failure.report,
                        "Could not load group memberships for Azure role assignments",
                    )
                });
                eyre::Ok(EntraUserAzureRoleAssignments {
                    rows,
                    membership_error,
                })
            },
        );
    }
}

fn is_loading<T>(result: &Loadable<T>) -> bool {
    matches!(result, Loadable::Loading { .. })
}

pub(crate) fn show_entra_user_azure_roles(app: &mut MyApp, ui: &mut Ui, user_id: EntraUserId) {
    app.load_entra_user_azure_roles(user_id, false);
    let Some(result) = app
        .azure
        .entra_user_permissions
        .azure_roles
        .get(&user_id)
        .and_then(|result| {
            loaded(
                ui,
                result,
                "Could not load Azure role assignments for this user.",
            )
        })
    else {
        return;
    };
    if let Some(error) = &result.membership_error {
        CollapsingHeader::new("Group memberships could not be loaded. Showing direct assignments.")
            .id_salt(user_id)
            .show(ui, |ui| DiagnosticReportView::new(error).show(ui));
    }

    let id = Id::new((
        "entra-user-azure-roles",
        app.azure.account_generation,
        user_id,
    ));
    let columns = [
        ReflectedTableColumn::new("Role", "role")
            .with_width(220.0)
            .groupable(),
        ReflectedTableColumn::new("Scope", "scope")
            .with_link("azure_scope")
            .groupable(),
        ReflectedTableColumn::new("Assigned through", "assigned_via")
            .with_width(125.0)
            .groupable(),
        ReflectedTableColumn::new("Group ID", "assigned_group")
            .with_link("assigned_group")
            .with_width(240.0),
        ReflectedTableColumn::new("Condition", "condition").with_width(140.0),
    ];
    if let Some(open) = ReflectedTable::new(id, &columns)
        .with_result_count()
        .show_explorable(
            ui,
            &result.rows,
            |row| id.with(&row.id),
            &mut app.azure.checkboxes,
        )
    {
        open.apply(app);
    }
}
