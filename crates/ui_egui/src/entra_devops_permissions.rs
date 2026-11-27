mod entra_devops_permission_identity;
mod entra_devops_permission_organization;
mod entra_devops_permission_row;
mod entra_devops_permission_subject;

pub(crate) use entra_devops_permission_organization::EntraDevOpsPermissionOrganization;

use self::entra_devops_permission_subject::EntraDevOpsPermissionSubject;
use crate::app::MyApp;
use crate::portal_page::PortalPage;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure_devops::AzureDevOpsEntraPermissionSubject;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganizationStore;
use cloud_terrastodon_azure_devops::scan_azure_devops_entra_permissions;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CollapsingHeader;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eyre::OptionExt;
use std::collections::HashMap;
use teamy_tracing::gui::DiagnosticReportView;

impl MyApp {
    pub(crate) fn load_entra_group_devops_permissions(
        &mut self,
        group_id: EntraGroupId,
        refresh: bool,
    ) {
        self.load_entra_devops_permissions(EntraDevOpsPermissionSubject::Group(group_id), refresh);
    }

    pub(crate) fn load_entra_user_devops_permissions(
        &mut self,
        user_id: EntraUserId,
        refresh: bool,
    ) {
        self.load_entra_devops_permissions(EntraDevOpsPermissionSubject::User(user_id), refresh);
    }

    fn load_entra_devops_permissions(
        &mut self,
        subject: EntraDevOpsPermissionSubject,
        refresh: bool,
    ) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        match subject.results(&self.azure) {
            Some(Loadable::Loading { .. }) => return,
            None | Some(Loadable::NotLoaded) => {}
            _ if !refresh => return,
            _ => {}
        }

        let tenant_id = authentication.tenant_id;
        let namespace = CloudTerrastodonAccountNamespace::Entra { tenant_id };
        let default = self
            .account_picker
            .catalogue
            .as_ref()
            .ok()
            .and_then(|catalogue| catalogue.resolve(None, Some(&namespace)).ok())
            .and_then(|profile| profile.azure_bearer_selection(AuthSource::Auto).ok());
        let devops_auth = self.ready_azure_authentication_for(default);
        let (description, request_subject) = match subject {
            EntraDevOpsPermissionSubject::Group(id) => (
                "Check Azure DevOps group permissions",
                AzureDevOpsEntraPermissionSubject::Group(id),
            ),
            EntraDevOpsPermissionSubject::User(id) => (
                "Check Azure DevOps user permissions",
                AzureDevOpsEntraPermissionSubject::User(id),
            ),
        };
        let _ = self.queue_entra_work(
            description,
            move |azure, load| *subject.results_mut(azure) = load,
            async move |executor| {
                let organizations = TrackedAzureDevOpsOrganizationStore::configured()
                    .list_for_tenant(tenant_id)
                    .await?;
                if organizations.is_empty() {
                    return Ok(Vec::new());
                }
                let authentication = devops_auth.ok_or_eyre(
                    "Set and sign in to the tenant's default account to check Azure DevOps permissions.",
                )?;
                let organizations = scan_azure_devops_entra_permissions(
                    organizations,
                    authentication,
                    request_subject,
                    refresh,
                    &executor,
                )
                .await?;
                Ok(organizations
                    .into_iter()
                    .map(EntraDevOpsPermissionOrganization::from_scan)
                    .collect())
            },
        );
    }
}

pub(crate) fn show_entra_group_devops_permissions(
    app: &mut MyApp,
    ui: &mut Ui,
    group_id: EntraGroupId,
) {
    show_entra_devops_permissions(app, ui, EntraDevOpsPermissionSubject::Group(group_id));
}

pub(crate) fn show_entra_user_devops_permissions(
    app: &mut MyApp,
    ui: &mut Ui,
    user_id: EntraUserId,
) {
    show_entra_devops_permissions(app, ui, EntraDevOpsPermissionSubject::User(user_id));
}

fn show_entra_devops_permissions(
    app: &mut MyApp,
    ui: &mut Ui,
    subject: EntraDevOpsPermissionSubject,
) {
    app.load_entra_devops_permissions(subject, false);
    let demo_mode = app.egui_config.demo_mode;
    ui.heading("Azure DevOps permissions");
    ui.label("Direct and parent-group grants visible to the tenant’s default account across its tracked organizations.");
    let mut navigate = None;
    match subject.results(&app.azure) {
        None | Some(Loadable::NotLoaded) => {}
        Some(Loadable::Loading { .. }) => {
            ui.spinner();
        }
        Some(Loadable::Failed { error, .. }) => {
            if ui.link("Manage accounts").clicked() {
                navigate = Some(PortalPage::Accounts);
            }
            DiagnosticReportView::new(error).show(ui);
        }
        Some(Loadable::Loaded { value, .. }) if value.is_empty() => {
            ui.label("No tracked Azure DevOps organizations are associated with this tenant.");
            if ui.link("Azure DevOps organizations").clicked() {
                navigate = Some(PortalPage::AzureDevOpsOrganizations);
            }
        }
        Some(Loadable::Loaded { value, .. }) => {
            ScrollArea::vertical().show(ui, |ui| {
                for organization in value {
                    CollapsingHeader::new(organization.organization.name())
                        .id_salt(&organization.organization)
                        .default_open(true)
                        .show(ui, |ui| {
                            show_organization_permissions(ui, organization, subject, demo_mode)
                        });
                }
            });
        }
    }
    if let Some(page) = navigate {
        app.navigate(page);
    }
}

fn show_organization_permissions(
    ui: &mut Ui,
    scan: &EntraDevOpsPermissionOrganization,
    subject: EntraDevOpsPermissionSubject,
    demo_mode: bool,
) {
    let identities = match &scan.result {
        Ok(identities) if identities.is_empty() => {
            ui.label(format!(
                "No matching Entra {} is visible in this organization.",
                subject.label()
            ));
            return;
        }
        Ok(identities) => identities,
        Err(error) => {
            DiagnosticReportView::new(error).show(ui);
            return;
        }
    };
    for result in identities {
        let identity = match result {
            Ok(identity) => identity,
            Err(error) => {
                DiagnosticReportView::new(error).show(ui);
                continue;
            }
        };
        let report = &identity.report;
        if !demo_mode {
            ui.strong(&identity.display_name);
        }
        ui.label(format!(
            "{} security namespaces checked",
            report.checked_namespace_count
        ));
        if report.parent_memberships_available {
            let label = format!("Parent groups ({})", report.parent_groups.len());
            if demo_mode {
                ui.label(label);
            } else {
                CollapsingHeader::new(label)
                    .id_salt((&report.subject, "parents"))
                    .show(ui, |ui| {
                        for group in &report.parent_groups {
                            ui.monospace(group.to_string());
                        }
                    });
            }
        } else {
            ui.label("Parent group memberships were unavailable.");
        }
        if !report.failed_namespaces.is_empty() {
            CollapsingHeader::new(format!(
                "{} security namespaces unavailable",
                report.failed_namespaces.len()
            ))
            .id_salt((&report.subject, "unavailable"))
            .show(ui, |ui| {
                for (namespace, failure) in &report.failed_namespaces {
                    ui.strong(namespace);
                    ui.label(failure);
                }
            });
        }
        if report.entries.is_empty() {
            ui.label("No explicit permission entries were found in the checked namespaces.");
            continue;
        }
        let mut columns = vec![
            ReflectedTableColumn::new("Namespace", "namespace")
                .with_width(145.0)
                .groupable(),
            ReflectedTableColumn::new("Security token", "token").with_width(160.0),
            ReflectedTableColumn::new("Assignment", "assignment")
                .with_width(110.0)
                .groupable(),
        ];
        if !demo_mode {
            columns.push(ReflectedTableColumn::new("Assignee", "assignee").with_width(170.0));
        }
        columns.extend([
            ReflectedTableColumn::new("Allow", "allow").with_width(180.0),
            ReflectedTableColumn::new("Deny", "deny"),
        ]);
        ui.push_id(&report.subject, |ui| {
            ReflectedTable::new("entra-devops-permissions", &columns)
                .without_selection()
                .with_search_hint("Search permissions")
                .with_result_count()
                .show_explorable(
                    ui,
                    &identity.rows,
                    |row| Id::new((&row.namespace_id, &row.token, &row.assignee)),
                    &mut HashMap::new(),
                );
        });
    }
}
