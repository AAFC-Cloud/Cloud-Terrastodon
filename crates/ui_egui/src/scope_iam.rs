use crate::app::MyApp;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use crate::scope_deny_assignment_row::ScopeDenyAssignmentRow;
use crate::scope_iam_comparison::compare_scope_iam;
use crate::scope_iam_section::ScopeIamSection;
use crate::scope_iam_snapshot::ScopeIamSnapshot;
use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use crate::scope_role_definition_row::ScopeRoleDefinitionRow;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_command::CancellationErrorExt;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use eframe::egui::Button;
use eframe::egui::Id;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use std::sync::Arc;
use std::time::Instant;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;
use tokio::runtime::Handle;
use tracing::warn;

pub(crate) fn scope_iam_ui(app: &mut MyApp, ui: &mut Ui, scope: &ScopeImpl) {
    let view = ui.id().with(("scope-iam", scope, app.account_generation));
    let section_key = view.with("section");
    let mut section = ui
        .data(|data| data.get_temp(section_key))
        .unwrap_or_default();
    let loading = matches!(app.scope_iam.get(scope), Some(Loadable::Loading { .. }));
    let refresh = ui
        .horizontal_wrapped(|ui| {
            for next in ScopeIamSection::ALL {
                ui.selectable_value(&mut section, next, next.label());
            }
            ui.add_enabled(!loading, Button::new("Refresh")).clicked()
        })
        .inner;
    ui.data_mut(|data| data.insert_temp(section_key, section));
    if refresh || !app.scope_iam.contains_key(scope) {
        load_scope_iam(app, scope, refresh);
    }

    match app.scope_iam.get(scope) {
        Some(Loadable::Loaded { value, .. }) => {
            let snapshot = Arc::clone(value);
            let id = view.with(section.label());
            let filter_key = id.with("filter");
            let mut filter: String = ui
                .data(|data| data.get_temp(filter_key))
                .unwrap_or_default();
            let hint = match section {
                ScopeIamSection::RoleAssignments => "Filter by principal ID, role, or scope",
                ScopeIamSection::Roles => "Filter roles",
                ScopeIamSection::DenyAssignments => "Filter deny assignments",
            };
            if ui
                .add(TextEdit::singleline(&mut filter).hint_text(hint))
                .changed()
            {
                ui.data_mut(|data| data.insert_temp(filter_key, filter.clone()));
            }
            let filter = filter.trim().to_lowercase();
            match section {
                ScopeIamSection::RoleAssignments => match &snapshot.assignments {
                    Ok(rows) => show_assignments(app, ui, id, rows, &filter),
                    Err(error) => DiagnosticReportView::new(error).show(ui),
                },
                ScopeIamSection::Roles => match &snapshot.roles {
                    Ok(rows) => show_roles(app, ui, id, rows, &filter),
                    Err(error) => DiagnosticReportView::new(error).show(ui),
                },
                ScopeIamSection::DenyAssignments => match &snapshot.denies {
                    Ok(rows) => show_denies(app, ui, id, rows, &filter),
                    Err(error) => DiagnosticReportView::new(error).show(ui),
                },
            }
        }
        Some(Loadable::Failed { error, .. }) => {
            DiagnosticReportView::new(error).show(ui);
        }
        _ => {
            ui.spinner();
        }
    }
}

pub(crate) fn load_scope_iam(app: &mut MyApp, scope: &ScopeImpl, refresh: bool) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    if matches!(app.scope_iam.get(scope), Some(Loadable::Loading { .. })) {
        return;
    }
    let account_generation = app.account_generation;
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let view_scope = scope.clone();
    let load_scope = scope.clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, Arc<ScopeIamSnapshot>>::new()
        .description("Load access control")
        .setter(move |app, value| {
            if app.account_generation == account_generation {
                let snapshot = value.as_loaded().cloned();
                app.scope_iam.insert(view_scope.clone(), value);
                if let Some(snapshot) = snapshot {
                    verify_scope_iam(app, authentication, &view_scope, refresh, snapshot);
                }
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            ScopeIamSnapshot::load(&executor, authentication, &load_scope, refresh)
                .await
                .map(Arc::new)
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = enqueue_result {
        app.scope_iam.insert(
            scope.clone(),
            Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            },
        );
    }
}

fn verify_scope_iam(
    app: &MyApp,
    authentication: AzureBearerAuthenticationSelection,
    scope: &ScopeImpl,
    refresh: bool,
    snapshot: Arc<ScopeIamSnapshot>,
) {
    if snapshot.assignments.is_err() && snapshot.roles.is_err() {
        return;
    }
    let executor = app.executor.clone();
    let work_cache = app.work_cache.clone();
    let comparison_scope = scope.clone();
    let work = app.work_tracker.spawn(
        app.executor.cancellation(),
        &Handle::current(),
        "Verify scoped Azure access control".to_owned(),
        async move |cancellation| {
            let comparison = async {
                let executor =
                    work_cache.job_executor(&executor.with_cancellation_scope(cancellation))?;
                compare_scope_iam(
                    &executor,
                    authentication,
                    &comparison_scope,
                    refresh,
                    snapshot.assignments.as_deref().ok(),
                    snapshot.roles.as_deref().ok(),
                )
                .await
            }
            .await;
            if let Err(error) = comparison {
                if error.is_cancellation() {
                    return Err(error);
                }
                let report = DiagnosticReport::new(error);
                warn!(
                    scope = %comparison_scope.expanded_form(),
                    account_id = %authentication.account_id,
                    error = report.as_error(),
                    "Could not verify scoped Azure access control"
                );
            }
            Ok(())
        },
    );
    if let Err(error) = work
        && !error.is_cancellation()
    {
        let report = DiagnosticReport::new(error);
        warn!(
            scope = %scope.expanded_form(),
            account_id = %authentication.account_id,
            error = report.as_error(),
            "Could not schedule scoped Azure access control verification"
        );
    }
}

fn show_assignments(
    app: &mut MyApp,
    ui: &mut Ui,
    id: Id,
    rows: &[ScopeRoleAssignmentRow],
    filter: &str,
) {
    let columns = [
        ReflectedTableColumn::new("Principal ID", "principal_id").with_width(174.0),
        ReflectedTableColumn::new("Type", "principal_type").with_width(100.0),
        ReflectedTableColumn::new("Role", "role").with_width(140.0),
        ReflectedTableColumn::new("Access", "inheritance").with_width(88.0),
        ReflectedTableColumn::new("Scope", "scope")
            .with_link("scope")
            .with_width(140.0),
        ReflectedTableColumn::new("Condition", "condition").with_width(110.0),
    ];
    let row_id = |row: &ScopeRoleAssignmentRow| id.with(&row.assignment.id);
    if let Some(row) = rows
        .iter()
        .find(|row| app.checkboxes.get(&row_id(row)) == Some(&true))
    {
        ui.collapsing("Assignment details", |ui| {
            ReflectedValue::new(row.assignment.as_ref()).show(ui)
        });
    }
    if rows.is_empty() {
        ui.label("No role assignments were returned for this scope.");
    }
    if let Some(target) = ReflectedTable::new(id, &columns).show(
        ui,
        rows.iter()
            .filter(|row| filter.is_empty() || row.matches(filter)),
        row_id,
        &mut app.checkboxes,
    ) {
        app.navigate_tab(target);
    }
}

fn show_roles(app: &mut MyApp, ui: &mut Ui, id: Id, rows: &[ScopeRoleDefinitionRow], filter: &str) {
    let columns = [
        ReflectedTableColumn::new("Role", "role").with_width(180.0),
        ReflectedTableColumn::new("Type", "kind").with_width(110.0),
        ReflectedTableColumn::new("Description", "description"),
    ];
    let row_id = |row: &ScopeRoleDefinitionRow| id.with(&row.definition.id);
    if let Some(row) = rows
        .iter()
        .find(|row| app.checkboxes.get(&row_id(row)) == Some(&true))
    {
        ui.collapsing("Role details", |ui| {
            ReflectedValue::new(row.definition.as_ref()).show(ui)
        });
    }
    if rows.is_empty() {
        ui.label("No role definitions were returned for this scope.");
    }
    if let Some(target) = ReflectedTable::new(id, &columns).show(
        ui,
        rows.iter()
            .filter(|row| filter.is_empty() || row.matches(filter)),
        row_id,
        &mut app.checkboxes,
    ) {
        app.navigate_tab(target);
    }
}

fn show_denies(
    app: &mut MyApp,
    ui: &mut Ui,
    id: Id,
    rows: &[ScopeDenyAssignmentRow],
    filter: &str,
) {
    let columns = [
        ReflectedTableColumn::new("Name", "name").with_width(180.0),
        ReflectedTableColumn::new("Access", "inheritance").with_width(88.0),
        ReflectedTableColumn::new("Scope", "scope").with_link("scope"),
        ReflectedTableColumn::new("Principals", "principal_count").with_width(80.0),
    ];
    let row_id = |row: &ScopeDenyAssignmentRow| id.with(&row.assignment.id);
    if let Some(row) = rows
        .iter()
        .find(|row| app.checkboxes.get(&row_id(row)) == Some(&true))
    {
        ui.collapsing("Deny assignment details", |ui| {
            ReflectedValue::new(row.assignment.as_ref()).show(ui)
        });
    }
    if rows.is_empty() {
        ui.label("No deny assignments were returned for this scope.");
    }
    if let Some(target) = ReflectedTable::new(id, &columns).show(
        ui,
        rows.iter()
            .filter(|row| filter.is_empty() || row.matches(filter)),
        row_id,
        &mut app.checkboxes,
    ) {
        app.navigate_tab(target);
    }
}
