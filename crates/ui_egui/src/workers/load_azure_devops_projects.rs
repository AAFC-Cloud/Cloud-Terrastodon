use crate::app::MyApp;
use crate::loadable_work::LoadableWorkBuilder;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use std::rc::Rc;
use tracing::info;

pub fn load_azure_devops_projects(app: &mut MyApp) {
    info!("Queueing work to fetch Azure DevOps projects...");
    LoadableWorkBuilder::<Vec<AzureDevOpsProject>>::new()
        .description("loading Azure DevOps projects")
        .setter(|app, data| app.azure_devops_projects = data.map(Rc::new))
        .work(async move {
            let org_url = get_default_organization_url().await?;
            let projects = fetch_all_azure_devops_projects(&org_url).await?;
            Ok(projects)
        })
        .build()
        .unwrap()
        .enqueue(app);
}
