use crate::app::MyApp;
use crate::portal_section_tabs::portal_section_tabs;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use crate::scope_iam_comparison::compare_scope_iam;
use crate::scope_iam_denies::load_scope_iam_denies;
use crate::scope_iam_denies::show_scope_iam_denies;
use crate::scope_iam_inventory::ScopeIamInventory;
use crate::scope_iam_principals::load_scope_iam_principals;
use crate::scope_iam_section::ScopeIamSection;
use crate::scope_iam_snapshot::ScopeIamSnapshot;
use crate::scope_role_assignments_view::show_check_access;
use crate::scope_role_assignments_view::show_role_assignments;
use crate::scope_role_definition_row::ScopeRoleDefinitionRow;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_command::CancellationErrorExt;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::sync::Arc;
use std::time::Instant;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;
use tokio::runtime::Handle;
use tracing::warn;

pub(crate) fn scope_iam_ui(app: &mut MyApp, ui: &mut Ui, scope: &ScopeImpl) {
    let view = ui
        .id()
        .with(("scope-iam", scope, app.azure.account_generation));
    let previous = app.azure.scope_iam.section(scope);
    let mut section = previous;
    ui.horizontal_wrapped(|ui| {
        portal_section_tabs(
            ui,
            &mut section,
            ScopeIamSection::ALL,
            ScopeIamSection::label,
        );
    });
    if section != previous {
        app.azure.scope_iam.select_section(scope, section);
    }
    let id = view.with(section.label());
    if section == ScopeIamSection::DenyAssignments {
        show_scope_iam_denies(app, ui, id, scope);
        return;
    }
    if matches!(app.azure.scope_iam.inventory, Loadable::NotLoaded) {
        load_scope_iam(app, scope, false);
    }

    let inventory = match &app.azure.scope_iam.inventory {
        Loadable::Loaded { value, .. } => value,
        Loadable::Failed { error, .. } => {
            DiagnosticReportView::new(error).show(ui);
            return;
        }
        Loadable::Loading { .. } | Loadable::NotLoaded => {
            ui.spinner();
            return;
        }
    };
    // A new scope needs only a local projection once the account inventory is loaded.
    let (snapshot, projected) = if let Some(snapshot) = app.azure.scope_iam.snapshots.get(scope) {
        (Arc::clone(snapshot), false)
    } else {
        let mut snapshot = inventory.at_scope(scope);
        app.azure.scope_iam.principals.apply(&mut snapshot);
        let snapshot = Arc::new(snapshot);
        app.azure
            .scope_iam
            .snapshots
            .insert(scope.clone(), Arc::clone(&snapshot));
        (snapshot, true)
    };
    let refresh = projected
        && app
            .azure
            .scope_iam
            .refresh_scope
            .take_if(|requested| requested == scope)
            .is_some();
    if projected
        && app.azure.scope_iam.compared.insert(scope.clone())
        && let Some(authentication) = app.ready_azure_authentication()
    {
        verify_scope_iam(app, authentication, scope, refresh, Arc::clone(&snapshot));
    }
    match section {
        ScopeIamSection::CheckAccess | ScopeIamSection::RoleAssignments => {
            match &snapshot.assignments {
                Ok(rows) => {
                    if projected || section != previous {
                        load_scope_iam_principals(app, &snapshot, refresh);
                    }
                    let pending = app.azure.scope_iam.principals.pending_for(rows);
                    if pending > 0 {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(if pending == 1 {
                                "Resolving 1 principal…".to_owned()
                            } else {
                                format!("Resolving {pending} principals…")
                            });
                        });
                    }
                    if section == ScopeIamSection::CheckAccess {
                        show_check_access(app, ui, id, rows);
                    } else {
                        show_role_assignments(app, ui, id, rows);
                    }
                }
                Err(error) => DiagnosticReportView::new(error).show(ui),
            }
        }
        ScopeIamSection::Roles => match &snapshot.roles {
            Ok(rows) => show_roles(app, ui, id, rows),
            Err(error) => DiagnosticReportView::new(error).show(ui),
        },
        ScopeIamSection::DenyAssignments => {}
    }
}

pub(crate) fn load_scope_iam(app: &mut MyApp, scope: &ScopeImpl, refresh: bool) {
    if app.azure.scope_iam.section(scope) == ScopeIamSection::DenyAssignments {
        load_scope_iam_denies(app, scope, refresh);
        return;
    }
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    if matches!(app.azure.scope_iam.inventory, Loadable::Loading { .. })
        || (!refresh && !matches!(app.azure.scope_iam.inventory, Loadable::NotLoaded))
    {
        return;
    }
    app.azure.scope_iam.refresh_scope = refresh.then(|| scope.clone());
    let account_generation = app.azure.account_generation;
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, Arc<ScopeIamInventory>>::new()
        .description("Load Azure RBAC inventory")
        .setter(move |app, value| {
            if let Some(azure) = app.azure_content_mut(account_generation) {
                if value.as_loaded().is_some() {
                    azure.scope_iam.snapshots.clear();
                    azure.scope_iam.compared.clear();
                }
                azure.scope_iam.inventory = value;
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            ScopeIamInventory::load(&executor, authentication, refresh)
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
        app.azure.scope_iam.inventory = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
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

fn show_roles(app: &mut MyApp, ui: &mut Ui, id: Id, rows: &[ScopeRoleDefinitionRow]) {
    let columns = [
        ReflectedTableColumn::new("Role", "role").with_width(180.0),
        ReflectedTableColumn::new("Type", "kind")
            .with_width(110.0)
            .groupable(),
        ReflectedTableColumn::new("Description", "description"),
    ];
    let row_id = |row: &ScopeRoleDefinitionRow| id.with(&row.definition.id);
    if let Some(row) = rows
        .iter()
        .find(|row| app.azure.checkboxes.get(&row_id(row)) == Some(&true))
    {
        ui.collapsing("Role details", |ui| {
            ReflectedValue::new(row.definition.as_ref()).show(ui)
        });
    }
    if rows.is_empty() {
        ui.label("No role definitions were returned for this scope.");
    }
    if let Some(target) = ReflectedTable::new(id, &columns)
        .with_search_hint("Filter roles")
        .show_explorable(ui, rows, row_id, &mut app.azure.checkboxes)
    {
        target.apply(app);
    }
}
