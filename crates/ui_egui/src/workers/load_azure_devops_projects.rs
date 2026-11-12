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
    let Some(org_url) = app.azure_devops_organization.clone() else {
        return;
    };
    info!("Queueing work to fetch Azure DevOps projects...");
    let authentication = app.authentication;
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    let work = LoadableWorkBuilder::<MyApp, Arc<Vec<Arc<AzureDevOpsProject>>>>::new()
        .description("loading Azure DevOps projects")
        .setter(|app, data| app.azure_devops_projects = data)
        .work(async move |scope| {
            let executor = executor.with_cancellation_scope(scope);
            let projects = AzureDevOpsProjectsListRequest {
                org_url: Cow::Borrowed(&org_url),
                auth_selection: (authentication.into()),
            }
            .execute(&executor)
            .await?;
            Ok(projects)
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
    if let Err(error) = work {
        app.azure_devops_projects = Loadable::Failed {
            error,
            started_at,
            finished_at: Instant::now(),
        };
    }
}
