use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsServiceEndpoint;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

pub struct AzureDevOpsServiceEndpointsListRequest<'a> {
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: AzureDevOpsProjectArgument<'a>,
}

pub fn fetch_all_azure_devops_service_endpoints<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
) -> AzureDevOpsServiceEndpointsListRequest<'a> {
    AzureDevOpsServiceEndpointsListRequest {
        org_url,
        project: project.into(),
    }
}

#[async_trait]
impl<'a> CacheableCommand for AzureDevOpsServiceEndpointsListRequest<'a> {
    type Output = Vec<AzureDevOpsServiceEndpoint>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            "service-endpoint",
            "list",
            &self.org_url.organization_name,
            &self.project.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "devops",
            "service-endpoint",
            "list",
            "--organization",
            &self.org_url.to_string(),
            "--project",
            &self.project.to_string(),
            "--output",
            "json",
        ]);
        cmd.cache(self.cache_key());

        let response = cmd.run::<Vec<AzureDevOpsServiceEndpoint>>().await?;
        Ok(response)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsServiceEndpointsListRequest<'a>, 'a);

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
                    // We will queue using project name and project ID to ensure the endpoint allows both
                    {
                        let project_name = project.name.clone();
                        let org_url = org_url.clone();
                        work.enqueue(async move {
                            fetch_all_azure_devops_service_endpoints(&org_url, &project_name).await
                        });
                    }
                    {
                        let project_id = project.id.clone();
                        let org_url = org_url.clone();
                        work.enqueue(async move {
                            fetch_all_azure_devops_service_endpoints(&org_url, &project_id).await
                        });
                    }
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
