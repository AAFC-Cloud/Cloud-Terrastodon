use crate::app::MyApp;
use crate::icons::page_icon;
use crate::management_group_inventory_row::ManagementGroupInventoryRow;
use crate::portal_page::PortalPage;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_azure::AzureManagementGroupListRequest;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

const COLUMNS: &[ReflectedTableColumn] = &[
    ReflectedTableColumn::new("Name", "group.display_name")
        .with_link("group.id")
        .with_icon(|_, _| Some(page_icon(PortalPage::ManagementGroups)))
        .with_width(200.0),
    ReflectedTableColumn::new("Group ID", "group_name")
        .with_identifier("group.id")
        .with_width(190.0),
    ReflectedTableColumn::new("Ancestors", "ancestors"),
];

/// Browse the selected tenant's management groups and their ancestors.
pub(crate) fn management_groups_ui(app: &mut MyApp, ui: &mut Ui) {
    if app.show_azure_authentication_required(ui) {
        return;
    }
    if matches!(app.azure.management_groups, Loadable::NotLoaded) {
        load_management_groups(app, false);
    }

    match &app.azure.management_groups {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.spinner();
        }
        Loadable::Failed { error, .. } => {
            ui.label("Could not load management groups");
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
            if ui.button("Sign in again").clicked() {
                app.open_selected_account_sign_in();
            }
        }
        Loadable::Loaded { value: groups, .. } => {
            let rows = groups
                .iter()
                .map(ManagementGroupInventoryRow::from)
                .collect::<Vec<_>>();
            if let Some(target) = ReflectedTable::new("management_groups", COLUMNS)
                .with_result_count()
                .with_search_hint("Search management groups")
                .show_explorable(
                    ui,
                    &rows,
                    |row| Id::new(("management_groups", &row.group.id)),
                    &mut app.azure.checkboxes,
                )
            {
                target.apply(app);
            }
        }
    }
}

pub(crate) fn load_management_groups(app: &mut MyApp, refresh: bool) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let account_generation = app.azure.account_generation;
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Load management groups")
        .setter(move |app, groups| {
            if let Some(azure) = app.azure_content_mut(account_generation) {
                azure.management_groups = groups;
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            AzureManagementGroupListRequest { authentication }
                .execute(&executor)
                .with_invalidation(refresh)
                .await
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = enqueue_result {
        app.azure.management_groups = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}
