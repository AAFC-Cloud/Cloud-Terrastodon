use crate::app::MyApp;
use crate::loadable::Loadable;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use std::rc::Rc;
use std::sync::Arc;
use tracing::info;

pub fn load_azure_devops_projects(app: &mut MyApp) {
    info!("Queueing work to fetch Azure DevOps projects...");
    let auth_context = app.auth_context.clone();
    let tenant_id = app.tenant_id;
    let work = LoadableWorkBuilder::<Vec<Arc<AzureDevOpsProject>>>::new()
        .description("loading Azure DevOps projects")
        .setter(|app, data| app.azure_devops_projects = data.map(Rc::new))
        .work(app.executor.cancellable_scope(move |executor| async move {
            let azure_devops_auth_context =
                AzureDevOpsAuthContext::for_tenant(&auth_context, tenant_id)?;
            let executor = azure_devops_auth_context.bind_executor(&executor)?;
            let org_url = get_default_organization_url(executor.cancellation()).await?;
            let projects =
                fetch_all_azure_devops_projects(&org_url, &azure_devops_auth_context, &executor)
                    .await?;
            Ok(projects)
        }))
        .build();
    match work {
        Ok(work) => work.enqueue(app),
        Err(error) => app.azure_devops_projects = Loadable::Failed(error),
    }
}
