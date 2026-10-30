use crate::app::MyApp;
use crate::loadable::Loadable;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use std::rc::Rc;
use tracing::info;

pub fn load_azure_devops_projects(app: &mut MyApp) {
    info!("Queueing work to fetch Azure DevOps projects...");
    let auth_context = app.auth_context.clone();
    let work = LoadableWorkBuilder::<Vec<AzureDevOpsProject>>::new()
        .description("loading Azure DevOps projects")
        .setter(|app, data| app.azure_devops_projects = data.map(Rc::new))
        .work(
            app.cancellation
                .cancellable_scope(move |cancellation| async move {
                    let org_url = get_default_organization_url(&cancellation).await?;
                    let azure_devops_auth_context = AzureDevOpsAuthContext::new(&auth_context)?;
                    let projects = fetch_all_azure_devops_projects(
                        &org_url,
                        &azure_devops_auth_context,
                        &cancellation,
                    )
                    .await?;
                    Ok(projects)
                }),
        )
        .build();
    match work {
        Ok(work) => work.enqueue(app),
        Err(error) => app.azure_devops_projects = Loadable::Failed(error),
    }
}
