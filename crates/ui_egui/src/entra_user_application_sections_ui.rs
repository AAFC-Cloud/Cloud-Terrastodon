use crate::app::MyApp;
use crate::entra_directory_ui::loaded;
use crate::entra_user_application_assignment_row::EntraUserApplicationAssignmentRow;
use crate::entra_user_application_assignment_row::assignment_role;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_azure::ConsentType;
use cloud_terrastodon_azure::EntraAppRoleAssignment;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CollapsingHeader;
use eframe::egui::Grid;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;

pub(crate) fn show_entra_user_application_ownership(
    app: &mut MyApp,
    ui: &mut Ui,
    user_id: EntraUserId,
) {
    app.load_entra_user_application_ownership(user_id, false);
    let Some(applications) = app
        .azure
        .entra_user_applications
        .ownership
        .get(&user_id)
        .and_then(|result| loaded(ui, result, "Could not load owned app registrations."))
    else {
        return;
    };

    let id = Id::new((
        "entra-user-application-ownership",
        app.azure.account_generation,
        user_id,
    ));
    let columns = [
        ReflectedTableColumn::new("Display name", "display_name")
            .with_width(280.0)
            .with_link("id")
            .with_icon(|_, _| Some(page_icon(PortalPage::EntraAppRegistrations))),
        ReflectedTableColumn::new("Application (client) ID", "app_id").with_width(290.0),
        ReflectedTableColumn::new("Object ID", "id").with_link("id"),
    ];
    if let Some(open) = ReflectedTable::new(id, &columns)
        .with_result_count()
        .show_explorable(
            ui,
            applications.iter().map(AsRef::as_ref),
            |application| id.with(application.id),
            &mut app.azure.checkboxes,
        )
    {
        open.apply(app);
    }
}

pub(crate) fn show_entra_user_application_assignments(
    app: &mut MyApp,
    ui: &mut Ui,
    user_id: EntraUserId,
) {
    app.load_entra_user_application_assignments(user_id, false);
    let Some(assignments) = app
        .azure
        .entra_user_applications
        .assignments
        .get(&user_id)
        .and_then(|result| loaded(ui, result, "Could not load application assignments."))
    else {
        return;
    };
    let id = Id::new((
        "entra-user-application-assignments",
        app.azure.account_generation,
        user_id,
    ));
    let selected = app
        .azure
        .entra_user_applications
        .selected_assignments
        .get(&user_id)
        .and_then(|selected_id| {
            assignments
                .iter()
                .find(|assignment| assignment.id == *selected_id)
        })
        .cloned();
    if let Some(assignment) = selected {
        show_assignment_details(app, ui, id, user_id, &assignment);
    }

    let Some(assignments) = app
        .azure
        .entra_user_applications
        .assignments
        .get(&user_id)
        .and_then(Loadable::as_loaded)
    else {
        return;
    };
    let rows: Vec<_> = assignments
        .iter()
        .map(|assignment| {
            let catalogue = app
                .azure
                .entra_user_applications
                .role_catalogues
                .get(&assignment.resource_id)
                .and_then(Loadable::as_loaded)
                .map(AsRef::as_ref);
            EntraUserApplicationAssignmentRow::new(assignment, catalogue, app.egui_config.demo_mode)
        })
        .collect();
    let columns = [
        ReflectedTableColumn::new("Application", "application")
            .with_width(240.0)
            .with_row_link()
            .with_icon(|_, _| Some(page_icon(PortalPage::EntraAppRegistrations)))
            .groupable(),
        ReflectedTableColumn::new("Assignment", "assignment_source")
            .with_width(110.0)
            .groupable(),
        ReflectedTableColumn::new("Granted to", "granted_to")
            .with_width(170.0)
            .with_link("group_id"),
        ReflectedTableColumn::new("App role", "role").with_width(180.0),
        ReflectedTableColumn::new("Assigned on", "assigned_on"),
    ];
    let mut activate = None;
    if let Some(open) = ReflectedTable::new(id, &columns)
        .with_result_count()
        .show_explorable_with_activation(
            ui,
            &rows,
            |row| id.with(&row.id),
            &mut app.azure.checkboxes,
            |row| activate = Some(row.id.clone()),
        )
    {
        open.apply(app);
    }
    if let Some(assignment_id) = activate {
        app.azure
            .entra_user_applications
            .selected_assignments
            .insert(user_id, assignment_id);
        ui.ctx().request_repaint();
    }
}

fn show_assignment_details(
    app: &mut MyApp,
    ui: &mut Ui,
    id: Id,
    user_id: EntraUserId,
    assignment: &EntraAppRoleAssignment,
) {
    app.load_entra_assignment_role_catalogue(assignment.resource_id, false);
    ui.horizontal(|ui| {
        ui.strong(&assignment.resource_display_name);
        if ui.button("Close details").clicked() {
            app.azure
                .entra_user_applications
                .selected_assignments
                .remove(&user_id);
            ui.ctx().request_repaint();
        }
    });
    let catalogue = app
        .azure
        .entra_user_applications
        .role_catalogues
        .get(&assignment.resource_id)
        .and_then(|result| loaded(ui, result, "Could not load application role definitions."));
    Grid::new(id.with(&assignment.id)).show(ui, |ui| {
        ui.label("App role");
        ui.label(assignment_role(assignment, catalogue.map(AsRef::as_ref)));
        ui.end_row();
        ui.label("Service principal object ID");
        ui.monospace(assignment.resource_id.to_string());
        ui.end_row();
        if let Some(catalogue) = catalogue {
            ui.label("Application (client) ID");
            ui.monospace(catalogue.app_id.to_string());
            ui.end_row();
        }
    });
    CollapsingHeader::new("Granted permissions")
        .id_salt(id.with((&assignment.id, "permissions")))
        .show(ui, |ui| {
            app.load_entra_assignment_permissions(assignment.resource_id, false);
            let lookup = ScrollArea::vertical()
                .id_salt(id.with("granted-permissions-scroll"))
                .max_height((ui.available_height() * 0.45).clamp(0.0, 240.0))
                .show(ui, |ui| {
                    show_granted_permissions(app, ui, id, user_id, assignment)
                })
                .inner;
            if let Some(resource_id) = lookup {
                app.load_entra_assignment_role_catalogue(resource_id, false);
            }
        });
    ui.separator();
}

fn show_granted_permissions(
    app: &MyApp,
    ui: &mut Ui,
    id: Id,
    user_id: EntraUserId,
    assignment: &EntraAppRoleAssignment,
) -> Option<EntraServicePrincipalObjectId> {
    let data = &app.azure.entra_user_applications;
    let mut lookup = None;
    ui.strong("App-only permissions");
    if let Some(permissions) = data
        .application_permissions
        .get(&assignment.resource_id)
        .and_then(|result| loaded(ui, result, "Could not load application permissions."))
    {
        if permissions.is_empty() {
            ui.weak("No application permissions.");
        } else {
            Grid::new(id.with("application-permissions"))
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Resource API");
                    ui.strong("App role");
                    ui.end_row();
                    for permission in permissions {
                        ui.label(&permission.resource_display_name);
                        let catalogue = data.role_catalogues.get(&permission.resource_id);
                        match catalogue {
                            Some(Loadable::Loading { .. }) => {
                                ui.spinner();
                            }
                            Some(Loadable::Loaded { value, .. }) => {
                                ui.label(assignment_role(permission, Some(value)));
                            }
                            Some(Loadable::Failed { .. }) => {
                                ui.label(assignment_role(permission, None))
                                    .on_hover_text("App role name unavailable");
                            }
                            _ if permission.app_role_id.is_nil() => {
                                ui.label(assignment_role(permission, None));
                            }
                            _ => {
                                if ui
                                    .link(permission.app_role_id.to_string())
                                    .on_hover_text("Look up app role name")
                                    .clicked()
                                {
                                    lookup = Some(permission.resource_id);
                                }
                            }
                        }
                        ui.end_row();
                    }
                });
        }
    }
    ui.strong("Delegated permissions");
    if let Some(permissions) = data
        .delegated_permissions
        .get(&assignment.resource_id)
        .and_then(|result| loaded(ui, result, "Could not load delegated permissions."))
    {
        let mut permissions = permissions
            .iter()
            .filter(|grant| grant.applies_to_user(user_id))
            .peekable();
        if permissions.peek().is_none() {
            ui.weak("No delegated permissions for this user.");
        } else {
            Grid::new(id.with("delegated-permissions"))
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Resource service principal ID");
                    ui.strong("Consent");
                    ui.strong("Scopes");
                    ui.end_row();
                    for grant in permissions {
                        ui.monospace(grant.resource_id.to_string());
                        ui.label(match grant.consent_type {
                            ConsentType::AllPrincipals => "Tenant-wide",
                            ConsentType::Principal => "This user",
                        });
                        ui.label(&grant.scope);
                        ui.end_row();
                    }
                });
        }
    }
    lookup
}
