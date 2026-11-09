use crate::app::MyApp;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

pub fn load_azure_devops_projects(app: &mut MyApp) {
    info!("Queueing work to fetch Azure DevOps projects...");
    let auth_context = app.auth_context.clone();
    let tenant_id = app.tenant_id;
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let started_at = Instant::now();
    let work = LoadableWorkBuilder::<MyApp, Arc<Vec<Arc<AzureDevOpsProject>>>>::new()
        .description("loading Azure DevOps projects")
        .setter(|app, data| app.azure_devops_projects = data)
        .work(async move |scope| {
            let executor = executor.with_cancellation_scope(scope);
            let azure_devops_auth_context =
                AzureDevOpsAuthContext::for_tenant(&auth_context, tenant_id)?;
            let executor = azure_devops_auth_context.bind_executor(&executor)?;
            let org_url = get_default_organization_url(executor.cancellation()).await?;
            let projects = fetch_all_azure_devops_projects(
                &org_url,
                azure_devops_auth_context.selection()?,
                &executor,
            )
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
