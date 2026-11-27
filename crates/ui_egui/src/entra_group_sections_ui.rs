use crate::app::MyApp;
use crate::entra_directory_ui::loaded;
use crate::entra_group_directory_object_row::EntraGroupDirectoryObjectRow;
use crate::entra_group_section::EntraGroupSection;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_azure::EntraGroupId;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::sync::Arc;

pub(crate) fn show_entra_group_section(
    app: &mut MyApp,
    ui: &mut Ui,
    group_id: EntraGroupId,
    section: EntraGroupSection,
) {
    let id = Id::new((
        "entra-group",
        app.azure.account_generation,
        group_id,
        section,
    ));
    match section {
        EntraGroupSection::Members | EntraGroupSection::Owners => {
            show_directory_objects(app, ui, id, group_id, section == EntraGroupSection::Members);
        }
        EntraGroupSection::RoleAssignments | EntraGroupSection::AzureRoleAssignments => {
            show_role_assignments(
                app,
                ui,
                id,
                group_id,
                section == EntraGroupSection::AzureRoleAssignments,
            );
        }
        EntraGroupSection::AuditLog => show_audit_log(app, ui, id, group_id),
        _ => {}
    }
}

fn show_directory_objects(
    app: &mut MyApp,
    ui: &mut Ui,
    id: Id,
    group_id: EntraGroupId,
    members: bool,
) {
    let inventory = &app.azure.entra_group_data;
    let (results, failure) = if members {
        (&inventory.members, "Could not load the group's members.")
    } else {
        (&inventory.owners, "Could not load the group's owners.")
    };
    let Some(rows) = results
        .get(&group_id)
        .and_then(|response| loaded(ui, response, failure))
    else {
        return;
    };
    let object_id = ReflectedTableColumn::new("Object ID", "object_id");
    let kind = ReflectedTableColumn::new("Type", "kind").groupable();
    let columns: &[ReflectedTableColumn] = if app.egui_config.demo_mode {
        &[
            object_id
                .with_link("principal_link")
                .with_icon(EntraGroupDirectoryObjectRow::icon),
            kind,
        ]
    } else {
        &[
            ReflectedTableColumn::new("Name", "name")
                .with_width(250.0)
                .with_link("principal_link")
                .with_icon(EntraGroupDirectoryObjectRow::icon),
            kind,
            object_id,
        ]
    };
    if let Some(open) = ReflectedTable::new(id, columns)
        .with_result_count()
        .show_explorable(
            ui,
            rows,
            |row| id.with(&row.object_id),
            &mut app.azure.checkboxes,
        )
    {
        open.apply(app);
    }
}

fn show_role_assignments(
    app: &mut MyApp,
    ui: &mut Ui,
    id: Id,
    group_id: EntraGroupId,
    azure_roles: bool,
) {
    let inventory = &app.azure.entra_group_data;
    let (results, failure) = if azure_roles {
        (
            &inventory.azure_roles,
            "Could not load Azure role assignments for this group.",
        )
    } else {
        (
            &inventory.directory_roles,
            "Could not load Entra role assignments for this group.",
        )
    };
    let Some(rows) = results
        .get(&group_id)
        .and_then(|response| loaded(ui, response, failure))
    else {
        return;
    };
    ui.weak("Direct assignments");
    let role = ReflectedTableColumn::new("Role", "role")
        .with_width(220.0)
        .groupable();
    let scope = ReflectedTableColumn::new("Scope", "scope");
    let columns: &[ReflectedTableColumn] = if azure_roles {
        &[
            role,
            scope.with_link("azure_scope").groupable(),
            ReflectedTableColumn::new("Condition", "condition").with_width(140.0),
        ]
    } else {
        &[role, scope.groupable()]
    };
    if let Some(open) = ReflectedTable::new(id, columns)
        .with_result_count()
        .show_explorable(ui, rows, |row| id.with(&row.id), &mut app.azure.checkboxes)
    {
        open.apply(app);
    }
}

fn show_audit_log(app: &mut MyApp, ui: &mut Ui, id: Id, group_id: EntraGroupId) {
    let Some(events) = app
        .azure
        .entra_group_data
        .audit_log
        .get(&group_id)
        .and_then(|response| loaded(ui, response, "Could not load the group's audit log."))
    else {
        return;
    };
    let columns = [
        ReflectedTableColumn::new("Time", "activity_date_time").with_width(220.0),
        ReflectedTableColumn::new("Activity", "activity_display_name"),
        ReflectedTableColumn::new("Result", "result")
            .with_width(110.0)
            .groupable(),
        ReflectedTableColumn::new("Category", "category")
            .with_width(150.0)
            .groupable(),
        ReflectedTableColumn::new("Service", "logged_by_service")
            .with_width(140.0)
            .groupable(),
    ];
    ReflectedTable::new(id, &columns)
        .initially_descending()
        .with_result_count()
        .show_explorable(
            ui,
            events.iter().map(Arc::as_ref),
            |event| id.with(&event.id),
            &mut app.azure.checkboxes,
        );
}
