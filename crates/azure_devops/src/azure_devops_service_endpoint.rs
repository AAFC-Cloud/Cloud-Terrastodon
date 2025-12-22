use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsServiceEndpoint;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_azure_devops_service_endpoints(
    org_url: &AzureDevOpsOrganizationUrl,
    project: &AzureDevOpsProjectName,
) -> eyre::Result<Vec<AzureDevOpsServiceEndpoint>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "service-endpoint",
        "list",
        "--organization",
        &org_url.to_string(),
        "--project",
        project,
        "--output",
        "json",
    ]);
    cmd.use_cache_behaviour(cloud_terrastodon_command::CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "devops",
            "service-endpoint",
            "list",
            &org_url.organization_name,
            project,
        ]),
        valid_for: Duration::MAX,
    });

    let response = cmd.run::<Vec<AzureDevOpsServiceEndpoint>>().await?;
    Ok(response)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::fetch_all_azure_devops_service_endpoints;
    use crate::prelude::get_default_organization_url;
    use crate::prelude::get_default_project_name;
    use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsServiceEndpoint;
    use cloud_terrastodon_command::ParallelFallibleWorkQueue;
    use itertools::Itertools;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        const JUST_ONE_PROJ: bool = true;
        if JUST_ONE_PROJ {
            let proj = get_default_project_name().await?;
            let service_endpoints =
                super::fetch_all_azure_devops_service_endpoints(&org_url, &proj).await?;

            println!("{:#?}", service_endpoints);
        } else {
            let projects = fetch_all_azure_devops_projects(&org_url).await?;
            let azure_devops_service_endpoints = {
                let mut work: ParallelFallibleWorkQueue<Vec<AzureDevOpsServiceEndpoint>> =
                    ParallelFallibleWorkQueue::new("azure devops service endpoints", 8);
                for project in projects.iter() {
                    let org_url = org_url.clone();
                    let project_name = project.name.clone();
                    work.enqueue(async move {
                        fetch_all_azure_devops_service_endpoints(&org_url, &project_name).await
                    });
                }
                work.join().await?.into_iter().flatten().collect_vec()
            };
            println!(
                "Found {} azure devops service endpoints across {} projects",
                azure_devops_service_endpoints.len(),
                projects.len()
            );
        }

        Ok(())
    }
}
