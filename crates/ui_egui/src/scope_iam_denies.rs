use crate::app::MyApp;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use crate::scope_deny_assignment_row::ScopeDenyAssignmentRow;
use cloud_terrastodon_azure::AzureDenyAssignmentListAtScopeRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::sync::Arc;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn load_scope_iam_denies(app: &mut MyApp, scope: &ScopeImpl, refresh: bool) {
    match app.azure.scope_iam.denies.get(scope) {
        Some(Loadable::Loading { .. }) => return,
        None | Some(Loadable::NotLoaded) => {}
        _ if !refresh => return,
        _ => {}
    }
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let account_generation = app.azure.account_generation;
    let scope_id = scope.clone();
    let request = AzureDenyAssignmentListAtScopeRequest {
        authentication,
        scope: scope.clone(),
    };
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let enqueue = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Load deny assignments")
        .setter(move |app, response| {
            if let Some(azure) = app.azure_content_mut(account_generation) {
                let rows = response.map(|assignments: Vec<_>| {
                    let selected_scope = scope_id.expanded_form();
                    Arc::new(
                        assignments
                            .into_iter()
                            .filter_map(|assignment| {
                                ScopeDenyAssignmentRow::applicable(assignment, &selected_scope)
                            })
                            .collect(),
                    )
                });
                azure.scope_iam.denies.insert(scope_id.clone(), rows);
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            request.execute(&executor).with_invalidation(refresh).await
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = enqueue {
        app.azure.scope_iam.denies.insert(
            scope.clone(),
            Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            },
        );
    }
}

pub(crate) fn show_scope_iam_denies(app: &mut MyApp, ui: &mut Ui, id: Id, scope: &ScopeImpl) {
    load_scope_iam_denies(app, scope, false);
    match app.azure.scope_iam.denies.get(scope) {
        Some(Loadable::Loaded { value, .. }) => {
            let rows = Arc::clone(value);
            show_denies(app, ui, id, &rows);
        }
        Some(Loadable::Failed { error, .. }) => {
            DiagnosticReportView::new(error).show(ui);
        }
        _ => {
            ui.spinner();
        }
    }
}

fn show_denies(app: &mut MyApp, ui: &mut Ui, id: Id, rows: &[ScopeDenyAssignmentRow]) {
    let columns = [
        ReflectedTableColumn::new("Name", "name").with_width(180.0),
        ReflectedTableColumn::new("Access", "inheritance")
            .with_width(88.0)
            .groupable(),
        ReflectedTableColumn::new("Scope", "scope")
            .with_link("scope")
            .groupable(),
        ReflectedTableColumn::new("Principals", "principal_count").with_width(80.0),
    ];
    let row_id = |row: &ScopeDenyAssignmentRow| id.with(&row.assignment.id);
    if let Some(row) = rows
        .iter()
        .find(|row| app.azure.checkboxes.get(&row_id(row)) == Some(&true))
    {
        ui.collapsing("Deny assignment details", |ui| {
            ReflectedValue::new(row.assignment.as_ref()).show(ui)
        });
    }
    if rows.is_empty() {
        ui.label("No deny assignments were returned for this scope.");
    }
    if let Some(target) = ReflectedTable::new(id, &columns)
        .with_search_hint("Filter deny assignments")
        .show_explorable_matching(
            ui,
            rows,
            row_id,
            &mut app.azure.checkboxes,
            ScopeDenyAssignmentRow::matches_principal_id,
        )
    {
        target.apply(app);
    }
}
