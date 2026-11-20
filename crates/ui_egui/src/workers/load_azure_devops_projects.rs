use crate::app::MyApp;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

pub fn load_azure_devops_projects(app: &mut MyApp) {
    load_azure_devops_projects_with_invalidation(app, false);
}

pub(crate) fn load_azure_devops_projects_with_invalidation(app: &mut MyApp, refresh: bool) {
    let account_generation = app.account_generation;
    let organization_generation = app.azure_devops_organization_picker.generation;
    let Some(org_url) = app.azure_devops_organization.clone() else {
        return;
    };
    let Some(auth_selection) = app.ready_azure_devops_authentication() else {
        return;
    };
    info!("Queueing work to fetch Azure DevOps projects...");
    let executor = app.executor.clone();
    let work_cache = app.work_cache.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, Arc<Vec<Arc<AzureDevOpsProject>>>>::new()
        .description("loading Azure DevOps projects")
        .setter(move |app, data| {
            if app.account_generation == account_generation
                && app.azure_devops_organization_picker.generation == organization_generation
            {
                app.azure_devops_projects = data;
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            AzureDevOpsProjectsListRequest {
                org_url: Cow::Borrowed(&org_url),
                auth_selection,
            }
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
        app.azure_devops_projects = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}
