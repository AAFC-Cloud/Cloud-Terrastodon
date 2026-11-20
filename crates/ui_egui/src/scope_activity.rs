use crate::app::MyApp;
use crate::portal_resource_scope::PortalResourceScope;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_azure::AzureActivityLogListRequest;
use cloud_terrastodon_azure::AzureActivityLogScope;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn scope_activity_ui(app: &mut MyApp, ui: &mut Ui, scope: &PortalResourceScope) {
    if matches!(
        app.scope_activity.get(scope),
        None | Some(Loadable::NotLoaded)
    ) {
        load_scope_activity(app, scope, false);
    }
    ui.weak("24-hour activity window · UTC");
    match app.scope_activity.get(scope) {
        Some(Loadable::Loaded { value, .. }) if value.is_empty() => {
            ui.label("No activity was returned for this period.");
        }
        Some(Loadable::Loaded { value, .. }) => {
            let columns = [
                ReflectedTableColumn::new("Time (UTC)", "event_timestamp").with_width(170.0),
                ReflectedTableColumn::new("Operation", "operation_name.value").with_width(248.0),
                ReflectedTableColumn::new("Status", "status.value").with_width(100.0),
                ReflectedTableColumn::new("Resource", "resource_id").with_link("resource_id"),
            ];
            if let Some(target) = ReflectedTable::new(("activity-log", scope), &columns)
                .initially_descending()
                .show(
                    ui,
                    value.iter().map(|event| event.as_ref()),
                    |event| Id::new(("activity-log-event", scope, &event.event_data_id)),
                    &mut app.checkboxes,
                )
            {
                app.navigate_tab(target);
            }
        }
        Some(Loadable::Failed { error, .. }) => {
            ui.label("Could not load activity for this scope.");
            let retry = ui.button("Retry").clicked();
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
            if retry {
                load_scope_activity(app, scope, true);
            }
        }
        None | Some(Loadable::NotLoaded | Loadable::Loading { .. }) => {
            ui.spinner();
        }
    }
}

pub(crate) fn load_scope_activity(app: &mut MyApp, scope: &PortalResourceScope, refresh: bool) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    if matches!(
        app.scope_activity.get(scope),
        Some(Loadable::Loading { .. })
    ) {
        return;
    }
    let request_scope = match scope {
        PortalResourceScope::ResourceGroup(id) => AzureActivityLogScope::ResourceGroup(id.clone()),
        PortalResourceScope::Subscription(id) => AzureActivityLogScope::Subscription(*id),
    };
    let account_generation = app.account_generation;
    let scope_id = scope.clone();
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let job = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Load activity log")
        .setter(move |app, response| {
            if app.account_generation == account_generation {
                app.scope_activity.insert(scope_id.clone(), response);
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            AzureActivityLogListRequest::last_day(authentication, request_scope)?
                .execute(&executor)
                .with_invalidation(refresh)
                .await
        })
        .build()
        .and_then(|work| {
            work.enqueue(
                &app.work_tracker.clone(),
                app.tx.clone(),
                app,
                &cancellation,
            )
        });
    if let Err(error) = job {
        app.scope_activity.insert(
            scope.clone(),
            Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            },
        );
    }
}
